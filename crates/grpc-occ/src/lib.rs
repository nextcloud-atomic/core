#[cfg(feature = "api")]
pub mod occ;

pub mod api {
    tonic::include_proto!("occ");
}
