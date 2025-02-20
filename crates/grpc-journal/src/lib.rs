#[cfg(feature = "api")]
pub mod journal_stream;

pub mod api {
    tonic::include_proto!("api");
}
