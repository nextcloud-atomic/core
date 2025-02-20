
pub mod types {
    use std::str::FromStr;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum ServiceStatus {
        ACTIVE,
        INACTIVE,
        ACTIVATING,
        DEACTIVATING,
        FAILED
    }

    impl TryFrom<String> for ServiceStatus {

        type Error = String;

        fn try_from(s: String) -> Result<Self, Self::Error> {
            s.parse()
        }
    }

    impl FromStr for ServiceStatus {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {

            match s {
                "\"active\"" => Ok(ServiceStatus::ACTIVE),
                "\"inactive\"" => Ok(ServiceStatus::INACTIVE),
                "\"activating\"" => Ok(ServiceStatus::ACTIVATING),
                "\"deactivating\"" => Ok(ServiceStatus::DEACTIVATING),
                "\"failed\"" => Ok(ServiceStatus::FAILED),
                s => Err(format!("Unexpected unit state: {s}"))
            }
        }
    }
}

#[cfg(feature = "backend")]
pub mod api {
    use zbus_systemd::{zbus, systemd1::ManagerProxy};
    use nca_error::NcaError;
    use libsystemd::daemon;
    use libsystemd::daemon::NotifyState;
    use zbus_systemd::zbus::fdo::PropertiesProxy;
    use zbus_systemd::znames::InterfaceName;
    use super::types::*;

    pub async fn get_service_status(name: String) -> Result<ServiceStatus, NcaError> {
        let conn = zbus::Connection::system().await?;
        let manager = ManagerProxy::new(&conn).await?;

        let unit_obj_path = manager.get_unit(name).await?;
        let props = PropertiesProxy::new(&conn, "org.freedesktop.systemd1", &unit_obj_path).await?;
        let active_state = props.get(InterfaceName::from_static_str("org.freedesktop.systemd1.Unit")?, "ActiveState").await?;

        active_state.to_string().parse()
            .map_err(NcaError::SystemdError)
    }

    pub fn sd_notify(state: &[NotifyState]) -> Result<(), NcaError> {
        daemon::notify(true, state)?;
        Ok(())
    }
}

