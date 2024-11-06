use std::fmt;
use std::fmt::Display;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum NcAtomicError {
    Generic(String),
    Unexpected(String),
    WeakPassword(usize, usize),
    MissingConfig(String),
    InvalidPath(String, String),
    ServerConfiguration(String),
    NotActivated(String),
    IOError(String),
    CryptoError(String),
}

impl fmt::Display for NcAtomicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cause: String = self.get_cause();
        let error_prefix = match *self {
            NcAtomicError::Generic(_) => "Generic Err",
            NcAtomicError::Unexpected(_) => "Unexpected Err",
            NcAtomicError::WeakPassword(_, _) => "WeakPassword Err",
            NcAtomicError::MissingConfig(_) => "Missing Config Err",
            NcAtomicError::InvalidPath(_, _) => "Invalid Path Err",
            NcAtomicError::ServerConfiguration(_) => "Configuration Err",
            NcAtomicError::NotActivated(_) => "Not Activated Err",
            NcAtomicError::CryptoError(_) => "Crypto Err",
            NcAtomicError::IOError(_) => "Input/Output Err",
        };
        write!(f, "{}: {}", error_prefix, cause)
    }
}

impl IntoResponse for NcAtomicError {
    fn into_response(self) -> Response {
        let cause: String = self.get_cause();
        let status = match self {
            NcAtomicError::Generic(_) | 
            NcAtomicError::Unexpected(_) |
            NcAtomicError::ServerConfiguration(_) |
            NcAtomicError::NotActivated(_) |
            NcAtomicError::IOError(_) |
            NcAtomicError::CryptoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NcAtomicError::WeakPassword(_, _) | 
            NcAtomicError::MissingConfig(_) | 
            NcAtomicError::InvalidPath(_, _) => StatusCode::BAD_REQUEST,
        };
        format!("status = {status}, message = {cause}").into_response()
    }
}

impl NcAtomicError {
    fn get_cause(&self) -> String {
        match self {
            NcAtomicError::Generic(cause) => cause.to_string(),
            NcAtomicError::Unexpected(cause) => cause.to_string(),
            NcAtomicError::ServerConfiguration(cause) => cause.to_string(),
            NcAtomicError::NotActivated(cause) => cause.to_string(),
            NcAtomicError::IOError(cause) => cause.to_string(),
            NcAtomicError::CryptoError(cause) => cause.to_string(),
            NcAtomicError::WeakPassword(length, minlength) => 
                format!(
                    "WeakPassword Err: The password needs to be of a minimum length of {}\
                    (only was {} characters long)",
                    minlength, length
                ).to_string(),
            NcAtomicError::MissingConfig(config) => 
                format!("Missing config: {}", config).to_string(),
            NcAtomicError::InvalidPath(path, msg) =>
                format!("The path '{}' is invalid. {}", path, msg).to_string(),
        }
    }
    
    pub fn new_server_config_error<D: Display>(s: D) -> NcAtomicError {
        NcAtomicError::ServerConfiguration(s.to_string())
    }
    
    pub fn new_unexpected_error<D: Display>(s: D) -> NcAtomicError {
        NcAtomicError::Unexpected(s.to_string())
    }
    
    pub fn new_io_error<D: Display>(s: D) -> NcAtomicError {
        NcAtomicError::IOError(s.to_string())
    }
    
    pub fn new_crypto_error<D: Display>(s: D) -> NcAtomicError {
        NcAtomicError::CryptoError(s.to_string())
    }
}