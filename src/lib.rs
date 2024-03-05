pub mod config;
pub mod crypto;
pub mod error;
#[cfg(feature = "ssr")]
pub mod secrets;
#[cfg(feature = "ssr")]
pub mod caddy;
#[cfg(feature = "ssr")]
pub mod templating;

pub const NCP_VERSION: &str = "2.0.0";
