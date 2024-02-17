use std::fmt;
use std::fmt::Formatter;
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct NcpError {
    msg: String
}

impl std::error::Error for NcpError {}

impl fmt::Display for NcpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "NcpError: {}", self.msg)
    }
}

impl From<String> for NcpError {
    fn from(value: String) -> Self {
        NcpError{ msg: value }
    }
}

impl From<&str> for NcpError {
    fn from(value: &str) -> Self {
        NcpError{ msg: value.to_string() }
    }
}

impl From<std::io::Error> for NcpError {
    fn from(value: std::io::Error) -> Self {
        NcpError{ msg: format!("{}({})", &value.kind(), value.to_string()) }
    }
}

impl Serialize for NcpError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(format!("NCPError({})", self.msg).as_str())
    }
}


