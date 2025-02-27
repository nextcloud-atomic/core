use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;
use std::fmt::Display;
use libsystemd::errors::SdError;

#[derive(Debug)]
pub enum NcaError {
    FaultySetup(String),
    SystemdError(String),
    Generic(String),
    Unexpected(String),
    WeakPassword(usize, usize),
    MissingConfig(String),
    InvalidPath(String, String),
    ServerConfiguration(String),
    NotActivated(String),
    IOError(String),
    CryptoError(String),
    NotReady(String)
}

// Allow the use of "{}" format specifier
impl fmt::Display for NcaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cause: String = self.get_cause();
        let error_prefix = match *self {
            NcaError::Generic(_) => "Generic Err",
            NcaError::Unexpected(_) => "Unexpected Err",
            NcaError::WeakPassword(_, _) => "WeakPassword Err",
            NcaError::MissingConfig(_) => "Missing Config Err",
            NcaError::InvalidPath(_, _) => "Invalid Path Err",
            NcaError::ServerConfiguration(_) => "Configuration Err",
            NcaError::NotActivated(_) => "Not Activated Err",
            NcaError::CryptoError(_) => "Crypto Err",
            NcaError::IOError(_) => "Input/Output Err",
            NcaError::SystemdError(_) => "Systemd Err",
            NcaError::FaultySetup(_) => "Faulty Setup Err",
            NcaError::NotReady(_) => "Not Ready Err",
        };
        write!(f, "{}: {}", error_prefix, cause)
    }
}
impl NcaError {
    fn get_cause(&self) -> String {
        match self {
            NcaError::Generic(cause) => cause.to_string(),
            NcaError::Unexpected(cause) => cause.to_string(),
            NcaError::ServerConfiguration(cause) => cause.to_string(),
            NcaError::NotActivated(cause) => cause.to_string(),
            NcaError::IOError(cause) => cause.to_string(),
            NcaError::CryptoError(cause) => cause.to_string(),
            NcaError::WeakPassword(length, minlength) =>
                format!(
                    "WeakPassword Err: The password needs to be of a minimum length of {}\
                    (only was {} characters long)",
                    minlength, length
                ).to_string(),
            NcaError::MissingConfig(config) =>
                format!("Missing config: {}", config).to_string(),
            NcaError::InvalidPath(path, msg) =>
                format!("The path '{}' is invalid. {}", path, msg).to_string(),
            NcaError::SystemdError(msg) => format!("Systemd Err: {}", msg).to_string(),
            NcaError::FaultySetup(msg) => format!("Faulty Setup Err: {}", msg).to_string(),
            NcaError::NotReady(msg) => format!("Not Ready Err: {}", msg).to_string(),
        }
    }

    pub fn new_server_config_error<D: Display>(s: D) -> NcaError {
        NcaError::ServerConfiguration(s.to_string())
    }

    pub fn new_unexpected_error<D: Display>(s: D) -> NcaError {
        NcaError::Unexpected(s.to_string())
    }

    pub fn new_io_error<D: Display>(s: D) -> NcaError {
        NcaError::IOError(s.to_string())
    }

    pub fn new_crypto_error<D: Display>(s: D) -> NcaError {
        NcaError::CryptoError(s.to_string())
    }

    pub fn new_missing_config_error<D: Display>(s: D) -> NcaError {
        NcaError::MissingConfig(s.to_string())
    }
}

// So that errors get printed to the browser?
impl IntoResponse for NcaError {
    fn into_response(self) -> Response {
        match self {
            NcaError::FaultySetup(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            NcaError::SystemdError(_) | NcaError::Generic(_) 
            | NcaError::Unexpected(_) | NcaError::ServerConfiguration(_) 
            | NcaError::IOError(_) | NcaError::CryptoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            NcaError::WeakPassword(_, _) | NcaError::InvalidPath(_, _) 
            | NcaError::NotActivated(_) | NcaError::MissingConfig(_) | NcaError::NotReady(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        }.into_response()

        // format!("status = {}, message = {}", status, error_message).into_response()
    }
}

#[cfg(feature = "tonic")]
use tonic::Status;
#[cfg(feature = "tonic")]
impl From<NcaError> for Status {
    fn from(value: NcaError) -> Self {

        match value {
            NcaError::Generic(_) => Status::internal(value.to_string()),
            NcaError::Unexpected(_) => Status::unknown(value.to_string()),
            NcaError::WeakPassword(_, _) => Status::invalid_argument(value.to_string()),
            NcaError::MissingConfig(_) => Status::invalid_argument(value.to_string()),
            NcaError::InvalidPath(_, _) => Status::invalid_argument(value.to_string()),
            NcaError::ServerConfiguration(_) => Status::internal(value.to_string()),
            NcaError::NotActivated(_) => Status::failed_precondition(value.to_string()),
            NcaError::IOError(_) => Status::internal(value.to_string()),
            NcaError::CryptoError(_) => Status::internal(value.to_string()),
            NcaError::SystemdError(_) => Status::internal(value.to_string()),
            NcaError::FaultySetup(_) => Status::internal(value.to_string()),
            NcaError::NotReady(_) => Status::failed_precondition(value.to_string()),
        }
    }
}

#[cfg(feature = "tonic")]
impl From<Status> for NcaError {
    fn from(value: Status) -> Self {
        NcaError::new_io_error(format!("Error during grpc call (status {}): {}", value.code(), value.message()))
    }
}


impl From<axum::http::uri::InvalidUri> for NcaError {
    fn from(err: axum::http::uri::InvalidUri) -> NcaError {
        NcaError::FaultySetup(err.to_string())
    }
}

impl From<zbus_systemd::zbus::Error> for NcaError {
    fn from(value: zbus_systemd::zbus::Error) -> Self {
        NcaError::SystemdError(value.to_string())
    }
}

impl From<SdError> for NcaError {
    fn from(value: SdError) -> Self {
        NcaError::SystemdError(value.to_string())
    }
}

impl From<zbus_systemd::znames::Error> for NcaError {
    fn from(value: zbus_systemd::znames::Error) -> Self {
        NcaError::SystemdError(value.to_string())
    }
}
impl From<zbus_systemd::zbus::fdo::Error> for NcaError {
    fn from(value: zbus_systemd::zbus::fdo::Error) -> Self {
        NcaError::SystemdError(value.to_string())
    }
}
