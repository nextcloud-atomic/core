use std::collections::HashMap;
use std::fs::File;
use axum::{Extension, Json};
use axum_extra::routing::TypedPath;
use url::Url;
use rand::Rng;
use serde::Deserialize;
use grpc_occ::occ::client::handle_occ_output;
use nca_error::NcaError;
use nca_system_api::systemd::{types::ServiceStatus, api::get_service_status};
use nca_caddy::{CaddyClient, config::builders};
use nca_system_api::occ::api::{set_nc_system_config, NcConfigValue};
use nca_api_model::{setup};
use crate::config::Config;
use paspio::entropy;
use tonic::transport::Channel;
use grpc_nca_system::api;
use grpc_nca_system::api::credentials_client::CredentialsClient;
use grpc_nca_system::api::nextcloud_client::NextcloudClient;
use grpc_nca_system::api::NextcloudConfig;
use grpc_nca_system::api::system_client::SystemClient;
use nca_api_model::setup::{CredentialsInitResponse, CredentialsInitRequest, Status};

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/service/*name")]
pub struct ServiceName {
    name: String,
}
#[cfg(not(feature = "mock-systemd"))]
pub(crate) async fn service_status(ServiceName{ name: svc_name }: ServiceName) -> Result<Json<ServiceStatus>, NcaError> {
    #[cfg(debug_assertions)]
    eprintln!("Retrieving service status for {svc_name}");
    let status = get_service_status(svc_name).await?;
    Ok(Json(status))
}

#[derive(Deserialize, Clone)]
pub struct NextcloudActivationParams {
    trusted_url: String,
}

pub(crate) async fn activate_endpoint_nextcloud(Extension(config): Extension<Config>, Json(params): Json<NextcloudActivationParams>) -> Result<Json<()>, NcaError> {
    #[cfg(debug_assertions)]
    eprintln!("Activate nextcloud endpoint");
    
    match &config.caddy_admin_socket {
        None => {
            eprintln!("ERROR: No caddy admin socket configured (Was CADDY_ADMIN_SOCKET set?)");
            return Err(NcaError::MissingConfig("CADDY_ADMIN_SOCKET".to_string()));
        },
        Some(caddy_socket_addr) => {
            #[cfg(not(feature = "mock-systemd"))]
            {
                if get_service_status("nextcloud-all-in-one.service".to_string()).await? != ServiceStatus::ACTIVE {
                    return Err(NcaError::NotReady("nextcloud is not running".to_string()))
                }
            }
            #[cfg(not(feature = "mock-occ"))]
            set_nc_default_domain(config.occ_channel, params.trusted_url.clone()).await?;
            
            let lb_cookie_secret: String = rand::rng()
                .sample_iter(rand::distr::Alphanumeric)
                .take(24).map(char::from)
                .collect();
            // let request_url = req.uri().host().ok_or(NcaError::Unexpected("URI missing from request".to_string()))?;
            let (server_cfg, lb_cookie_value) = builders::create_nextcloud_server_json(params.trusted_url, lb_cookie_secret);
            println!("cookie for admin/NC toggle: {lb_cookie_value}");
            let cfg = serde_json::to_string(&HashMap::from([("nextcloud", server_cfg)]))
                .expect("Failed to create nextcloud server config");
            let caddy = CaddyClient::new(caddy_socket_addr)
                .map_err(|e| NcaError::new_server_config_error(format!("Failed to setup caddy client at socket '{caddy_socket_addr}': {e:?}")))?;
            caddy.set_caddy_servers(cfg).await
                .map_err(|e| NcaError::new_io_error(format!("Failed to configure caddy at socket '{caddy_socket_addr}: {e:?}")))?;
        }
    }

    #[cfg(not(feature = "mock-fs"))]
    {
        File::create(format!("{}/system/setup_complete", config.config_path))
            .map_err(|e| NcaError::new_server_config_error(format!("Failed to create setup completion file: {e:?}")))?;
    }
    Ok(Json(()))
}

async fn add_nc_trusted_domain(occ_channel: Channel, domain: String) -> Result<String, NcaError> {
    use nca_system_api::occ::api::{NcConfigValue, set_nc_system_config};
    let response = set_nc_system_config(occ_channel,
                                            "trusted_domains".to_string(),
                                            Some(11),
                                            NcConfigValue::String(domain))
        .await?;

    handle_occ_output(response).await
        .map(|_| "occ command terminated successfully.".to_string())
}

async fn set_nc_default_domain(occ_channel: Channel, domain: String) -> Result<String, NcaError> {
    add_nc_trusted_domain(occ_channel.clone(), domain.clone()).await?;
    let response = set_nc_system_config(occ_channel.clone(),
                                        "overwrite.cli.url".to_string(),
                                        None,
                                        // TODO: get protocol from nc config
                                        NcConfigValue::String(format!("https://{domain}/")))
        .await?;

    handle_occ_output(response).await?;
    let response = set_nc_system_config(occ_channel,
                                        "overwritehost".to_string(),
                                        None,
                                        NcConfigValue::String(domain.to_string()))
        .await?;

    handle_occ_output(response).await
        .map(|_| "nextcloud was successfully configured".to_string())
}



fn check_is_secure_password(pw: &str) -> bool {
    if pw.is_empty() {
        return false;
    }
    entropy(pw) >= 130.0
}

pub async fn configure_nextcloud_atomic(Extension(config): Extension<Config>, Json(params): Json<setup::ServicesConfig>) -> Result<(), NcaError> {
    
    let url = Url::parse(format!("https://{}:80/", params.nextcloud_domain).as_str())
        .map_err(|e| NcaError::FaultySetup(format!("Failed to parse nextcloud domain: {e:?}")))?;
    if url.host_str()
        .ok_or(NcaError::FaultySetup("Failed to parse nextcloud domain (couldn't get host)".to_string()))? != params.nextcloud_domain.as_str() {
        return Err(NcaError::FaultySetup(format!("{} is not a valid nextcloud domain", params.nextcloud_domain)));
    }

    if !check_is_secure_password(&params.nextcloud_password) {
        return Err(NcaError::FaultySetup("The nextcloud admin password is too weak!".to_string()));
    }
    #[cfg(not(feature = "mock-systemd"))]
    {
        let channel = config.nca_system_channel;
        let mut nc_client = NextcloudClient::new(channel.clone());
        let mut system_client = SystemClient::new(channel.clone());

        let _nc_cfg = nc_client.update_config(
            tonic::Request::new(NextcloudConfig {
                domain: Some(params.nextcloud_domain),
                admin_password: Some(params.nextcloud_password)
            })
        ).await?.into_inner();
        
        let _result = system_client.unlock_from_systemd_credentials(
            tonic::Request::new(api::Empty{})
        ).await?;
    }

    Ok(())
}

pub async fn complete_credentials_setup(Extension(config): Extension<Config>) -> Result<Json<Status>, NcaError> {

    #[cfg(not(feature = "mock-systemd"))]
    {
        let mut client = CredentialsClient::new(config.nca_system_channel);

        let result = client.complete_setup(tonic::Request::new(api::Empty{})).await?.into_inner();
        Ok(Json(Status {
            status: result.status_text
        }))
    }

    #[cfg(feature = "mock-systemd")]
    {
        Ok(Json(Status {
            status: "success".to_string()
        }))
    }
}

pub async fn generate_credentials(Extension(config): Extension<Config>, Json(params): Json<CredentialsInitRequest>) -> Result<Json<CredentialsInitResponse>, NcaError> {

    if !check_is_secure_password(&params.primary_password) {
        return Err(NcaError::FaultySetup("The nextcloud admin password is too weak!".to_string()));
    }

    #[cfg(not(feature = "mock-systemd"))]
    {
        let mut client = CredentialsClient::new(config.nca_system_channel);

        let credentials = client.initialize_credentials(
            tonic::Request::new(params.primary_password.into())
        ).await?.into_inner();

        Ok(Json(CredentialsInitResponse {
            disk_encryption_password: credentials.disk_encryption_recovery_password,
            backup_password: credentials.backup_password,
            salt: credentials.salt
        }))
    }
    
    #[cfg(feature = "mock-systemd")]
    {
        Ok(Json(CredentialsInitResponse {
            disk_encryption_password: "fakepassword".to_string(),
            backup_password: "fakepassword".to_string(),
            salt: "fakesalt".to_string()
        }))
    }
}

pub async fn hard_reset_nextcloud(Extension(config): Extension<Config>) -> Result<Json<Status>, NcaError> {

    #[cfg(not(feature = "mock-systemd"))]
    {
        let mut client = NextcloudClient::new(config.nca_system_channel);
        client.hard_reset(tonic::Request::new(api::Empty{}))
            .await?
            .into_inner();
        Ok(Json(Status {
            status: "successfully performed Nextcloud hard reset".to_string()
        }))
    }
}

#[cfg(feature = "mock-systemd")]
pub mod mock {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use axum::extract::State;
    use axum::Json;
    use nca_error::NcaError;
    use nca_system_api::systemd::types::ServiceStatus;
    use crate::api_routes::ServiceName;


    #[derive(Debug, Clone)]
    pub(crate) struct ServiceMockState {
        pub(crate) service_status_request_count: Arc<Mutex<i32>>,
        pub(crate) target_states: HashMap<String, ServiceStatus>
    }

    pub(crate) async fn service_status(ServiceName{ name: svc_name}: ServiceName, State(state): State<ServiceMockState>) -> Result<Json<ServiceStatus>, NcaError> {
        #[cfg(debug_assertions)]
        eprintln!("Retrieving service status for {}", svc_name);

        let mut counter = state.service_status_request_count.lock().expect("mutex was poisoned");
        *counter += 1;
        let requests_until_startup = 5;
        if *counter < requests_until_startup {
            eprintln!("Services will be active in {} requests", requests_until_startup - *counter);
        } else {
            eprintln!("Services are active");
        }
        match *counter {
            i if i < requests_until_startup => Ok(Json(ServiceStatus::ACTIVATING)),
            _ => Ok(Json(state.target_states
                .get(&svc_name)
                .unwrap_or(&ServiceStatus::ACTIVATING).clone())),
        }
    }

}