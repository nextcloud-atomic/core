use grpc_occ::occ::client::run_occ_client;
use nca_error::NcaError;

#[tokio::main]
async fn main() -> Result<(), NcaError> {
    let mut args = std::env::args();
    args.next().unwrap();

    //let socket_path = args.next().expect("Missing required argument: socket path");
    let socket_path = std::env::var("OCC_SERVER_SOCKET")
        .expect("Variable OCC_SERVER_SOCKET is not set");
    let occ_args = args.collect::<Vec<String>>();
    
    run_occ_client(socket_path, occ_args).await
}