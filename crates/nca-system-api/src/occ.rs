
#[cfg(feature = "backend")]
pub mod api {
    use std::path::PathBuf;
    use tonic::Streaming;
    use grpc_occ::api::{Command, CommandOutput};
    use grpc_occ::api::occ_client::OccClient;
    use grpc_occ::occ::client::{get_socket_channel, run_occ_client};
    use nca_error::NcaError;

    pub enum NcConfigValue {
        String(String),
        Bool(bool),
        Int(i32)
    }
    
    impl NcConfigValue {
        fn get_args(self) -> (String, String) {
            match self {
                NcConfigValue::Bool(v) => ("bool".to_string(), format!("{v}")),
                NcConfigValue::Int(v) => ("int".to_string(), format!("{v}")),
                NcConfigValue::String(v) => ("string".to_string(), v),
            }
        }
    }
    
    pub async fn set_nc_system_config(occ_socket_path: String, key: String, index: Option<usize>, value: NcConfigValue) -> Result<Streaming<CommandOutput>, NcaError> {
        let (type_arg, value_arg) = value.get_args();
        let mut args: Vec<String> = vec!["config:system:set", "--type", &type_arg, "--value", &value_arg, &key]
            .into_iter().map(String::from).collect();
        if let Some(pos) = index {
            args.push(format!("{pos}"))
        }
        
        let channel = get_socket_channel(
            PathBuf::from(occ_socket_path),
            "https://occ.nextcloudatomic.local".to_string()
        ).await?;

        let mut client = OccClient::new(channel);
        let response = client.exec(Command{arguments: args}).await?
            .into_inner();
        
        Ok(response)
    }
    
    
}