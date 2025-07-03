#[cfg(feature = "api")]
pub mod server;
#[cfg(feature = "client")]

pub mod client;

pub mod api {
    tonic::include_proto!("api");
}
