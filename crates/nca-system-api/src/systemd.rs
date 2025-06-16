
pub mod types {
    use std::str::FromStr;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    use std::io::Write;
    use std::process::Stdio;
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
        let props = PropertiesProxy::new(&conn, "org.freedesktop.systemd1", unit_obj_path).await?;
        let active_state = props.get(InterfaceName::from_static_str("org.freedesktop.systemd1.Unit")?, "ActiveState").await?;

        active_state.to_string().parse()
            .map_err(NcaError::SystemdError)
    }
    
    pub async fn restart_service(name: String) -> Result<(), NcaError> {
        let conn = zbus::Connection::system().await?;
        let manager = ManagerProxy::new(&conn).await?;
        manager.restart_unit(name, "direct".to_string()).await
            .map_err(|e| NcaError::SystemdError(format!("{:?}", e)))?;
        Ok(())
    }
    
    pub async fn start_service(name: String) -> Result<(), NcaError> {
        #[cfg(debug_assertions)]
        eprintln!("Starting service {name} ...");
        let conn = zbus::Connection::system().await?;
        let manager = ManagerProxy::new(&conn).await?;
        manager.start_unit(name, "replace".to_string()).await
            .map_err(|e| NcaError::SystemdError(format!("{:?}", e)))?;
        Ok(())
    }

    pub fn sd_notify(state: &[NotifyState]) -> Result<(), NcaError> {
        daemon::notify(true, state)?;
        Ok(())
    }

    pub async fn set_systemd_credential(value: String, path: String, name: Option<String>) -> Result<(), NcaError> {
        #[cfg(debug_assertions)]
        eprintln!("Creating systemd credential at path: {path}");
        let mut cmd = std::process::Command::new("/usr/bin/systemd-creds");
        cmd.arg("encrypt");
        if let Some(cred_name) = name {
            cmd.arg(format!("--name={cred_name}"));
        }
        let mut proc = cmd.args(["-", &path]).stdin(Stdio::piped()).spawn()
            .map_err(|e| NcaError::IOError(format!("Failed to run systemd-creds: {:?}", e)))?;
        match proc.stdin.as_mut() {
            None => Err(NcaError::IOError("Failed to get stdin of systemd-creds".to_string())),
            Some(stdin) => {
                #[cfg(debug_assertions)]
                eprintln!("Writing to child process stdin ...");
                stdin.write_all(value.as_bytes())
                    .map_err(|e|  NcaError::IOError(format!("Failed to create systemd credential: {:?}", e)))
            }
        }?;
        // let mut stdin = proc.stdin.take()
        //     .ok_or(NcaError::IOError("Failed to get stdin of systemd-creds command".to_string()))?;
        // stdin.write_fmt(format_args!( "{}", value))
        //     .map_err(|e|  NcaError::IOError(format!("Failed to create systemd credential: {:?}", e)))?;
        #[cfg(debug_assertions)]
        eprintln!("Waiting for child process to finish ...");
        let out = proc.wait_with_output().map_err(|e| NcaError::IOError(format!("Failed to run systemd-creds: {:?}", e)))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(NcaError::IOError(format!("Failed to create systemd credentials (exit code: {:?}): {}", out.status.code(), String::from_utf8_lossy(&out.stderr))))
        }
    }
}

