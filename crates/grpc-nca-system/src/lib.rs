#[cfg(feature = "api")]
pub mod server;
#[cfg(feature = "api")]
pub mod crypto;

pub mod api {
    tonic::include_proto!("nca_system");
    
    impl From<String> for PrimaryPassword {
        fn from(value: String) -> Self {
            Self {
                value: value.to_owned(),
            }
        }
    }
}