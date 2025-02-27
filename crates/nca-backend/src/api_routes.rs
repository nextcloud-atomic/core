use axum::Json;
use axum_extra::routing::TypedPath;
use serde::Deserialize;
use nca_error::NcaError;
use nca_system_api::api::get_service_status;
use nca_system_api::types::ServiceStatus;

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

#[cfg(feature = "mock")]
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