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
use nca_system_api::systemd::api::{set_systemd_credential, start_service};
use nca_api_model::{setup};
use crate::config::Config;
use paspio::entropy;
use nca_api_model::setup::{CredentialsConfig, CredentialsInitRequest};
use crate::crypto::{b32_encode, create_key_from_pass, derive_key, generate_salt};

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/service/*name")]
pub struct ServiceName {
    name: String,
}
#[cfg(not(feature = "mock-systemd"))]
pub(crate) async fn service_status(ServiceName{ name: svc_name }: ServiceName) -> Result<Json<ServiceStatus>, NcaError> {
    #[cfg(debug_assertions)]
    eprintln!("Retrieving service status for {}", svc_name);
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
            set_nc_default_domain(config.occ_socket, params.trusted_url.clone()).await?;
            
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

async fn add_nc_trusted_domain(occ_socket_path: String, domain: String) -> Result<String, NcaError> {
    use nca_system_api::occ::api::{NcConfigValue, set_nc_system_config};
    let response = set_nc_system_config(occ_socket_path,
                                            "trusted_domains".to_string(),
                                            Some(11),
                                            NcConfigValue::String(domain))
        .await?;

    handle_occ_output(response).await
        .map(|_| "occ command terminated successfully.".to_string())
}

async fn set_nc_default_domain(occ_socket_path: String, domain: String) -> Result<String, NcaError> {
    add_nc_trusted_domain(occ_socket_path.clone(), domain.clone()).await?;
    let response = set_nc_system_config(occ_socket_path.clone(),
                                        "overwrite.cli.url".to_string(),
                                        None,
                                        // TODO: get protocol from nc config
                                        NcConfigValue::String(format!("https://{domain}/")))
        .await?;

    handle_occ_output(response).await?;
    let response = set_nc_system_config(occ_socket_path,
                                        "overwritehost".to_string(),
                                        None,
                                        NcConfigValue::String(format!("{domain}")))
        .await?;

    handle_occ_output(response).await
        .map(|_| "nextcloud was successfully configured".to_string())
}



fn check_is_secure_password(pw: &str) -> bool {
    if pw.is_empty() {
        return false;
    }
    entropy(&pw) >= 130.0
}

pub async fn configure_nextcloud_atomic(Extension(config): Extension<Config>, Json(params): Json<setup::NcAtomicInitializationConfig>) -> Result<(), NcaError> {
    
    let url = Url::parse(format!("https://{}:80/", params.services.nextcloud_domain).as_str())
        .map_err(|e| NcaError::FaultySetup(format!("Failed to parse nextcloud domain: {e:?}")))?;
    if url.host_str()
        .ok_or(NcaError::FaultySetup("Failed to parse nextcloud domain (couldn't get host)".to_string()))? != params.services.nextcloud_domain.as_str() {
        return Err(NcaError::FaultySetup(format!("{} is not a valid nextcloud domain", params.services.nextcloud_domain)));
    }

    // if !check_is_secure_password(&params.services.nextcloud_password) {
    //     return Err(NcaError::FaultySetup("The nextcloud admin password is too weak!".to_string()));
    // }
    
    // let disk_encryption_password = 

    #[cfg(not(feature = "mock-systemd"))]
    {
        set_systemd_credential(params.services.nextcloud_domain.clone(), format!("{}/nc-aio/credentials/nc_domain.txt", config.config_path), None).await?;
        // set_systemd_credential(params.services.nextcloud_password.clone(), format!("{}/nc-aio/credentials/nc_password.txt", config.config_path), None).await?;
        // set_systemd_credential(salt.clone(), format!("{}/credentials/ncatomic_salt.txt", config.config_path), None).await?;
        start_service("nca-unlock.service".to_string()).await?;
    }

    Ok(())
}

pub async fn generate_credentials(Extension(config): Extension<Config>, Json(params): Json<CredentialsInitRequest>) -> Result<Json<CredentialsConfig>, NcaError> {

    if !check_is_secure_password(&params.nextcloud_admin_password) {
        return Err(NcaError::FaultySetup("The nextcloud admin password is too weak!".to_string()));
    }
    if !check_is_secure_password(&params.primary_password) {
        return Err(NcaError::FaultySetup("The nextcloud admin password is too weak!".to_string()));
    }

    // let salt: String = rand::rng()
    //     .sample_iter(rand::distr::Alphanumeric)
    //     .take(12).map(char::from)
    //     .collect();
    // let disk_encryption_password =
    
    let salt = generate_salt();
    let salt_b32 = b32_encode(&salt);
    let primary_key = create_key_from_pass(&salt, params.primary_password.clone());
    let disk_encryption_password = derive_key(&primary_key, &salt, "NCA_DISK_ENCRYPTION".to_string())
        .map_err(|e| NcaError::CryptoError(format!("Failed to derive key from password: {e:?}")))?;
    let disk_encryption_password_b32 = b32_encode(&disk_encryption_password);
    let backup_password = derive_key(&primary_key, &salt, "NCA_BACKUP_ENCRYPTION".to_string())
        .map_err(|e| NcaError::CryptoError(format!("Failed to derive key from password: {e:?}")))?;
    let backup_password_b32 = b32_encode(&backup_password);

    #[cfg(not(feature = "mock-systemd"))]
    {
        // set_systemd_credential(params.services.nextcloud_domain.clone(), format!("{}/nc-aio/credentials/nc_domain.txt", config.config_path), None).await?;
        set_systemd_credential(params.nextcloud_admin_password.clone(), format!("{}/nc-aio/credentials/nc_password.txt", config.config_path), None).await?;
        // todo: add disk encryption password
        set_systemd_credential(backup_password_b32.clone(), format!("{}/credentials/ncatomic_backup_password.txt", config.config_path), None).await?;
        set_systemd_credential(salt_b32.clone(), format!("{}/credentials/ncatomic_salt.txt", config.config_path), None).await?;
    }
    Ok(Json(CredentialsConfig {
        primary_password: params.primary_password,
        salt: salt_b32,
        disk_encryption_password: disk_encryption_password_b32,
        backup_password: backup_password_b32,
    }))
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