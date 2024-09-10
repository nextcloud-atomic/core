#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod crypto;
pub mod error;
#[cfg(feature = "ssr")]
pub mod secrets;
#[cfg(feature = "ssr")]
pub mod templating;

pub const NCP_VERSION: &str = "2.0.0";
