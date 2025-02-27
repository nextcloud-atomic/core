
pub mod server {
    use std::io::Read;
    use std::thread;
    use tokio::sync::mpsc::Sender;
    use tonic::{Request, Response, Status};
    use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
    use crate::api::{Command, CommandOutput, OutputType};
    use crate::api::occ_server::Occ;

    pub struct OccService {
        container_cmd: String,
        container_name: String,
    }

    impl OccService {
        pub fn new(container_cmd: String, container_name: String) -> Self {
            Self { container_cmd, container_name }
        }
        #[cfg(not(feature = "mock"))]
        fn base_command(&self) -> [&str; 7] {
            ["exec", "-it", "-u", "www-data", &self.container_name, "php", "occ"]
        }

        #[cfg(feature = "mock")]
        fn base_command(&self) -> [&str; 7] {
            [&self.container_cmd, "exec", "-it", "-u", "www-data", &self.container_name, "php occ"]
        }
    }

    #[tonic::async_trait]
    impl Occ for OccService {
        type ExecStream = ReceiverStream<Result<CommandOutput, Status>>;

        async fn exec(&self, request: Request<Command>) -> Result<Response<Self::ExecStream>, Status> {
            #[cfg(debug_assertions)]
            println!("Received exec request: {request:?}");

            let args = request.into_inner().arguments;


            let (tx, rx) = tokio::sync::mpsc::channel(100);

            #[cfg(not(feature = "mock"))]
            let mut cmd = std::process::Command::new(&self.container_cmd);
            #[cfg(feature = "mock")]
            let mut cmd = {
                let mut cmd = std::process::Command::new("echo");
                cmd.args([&self.container_cmd]);
                cmd
            };
            cmd.args(self.base_command()).args(args);
            
            #[cfg(debug_assertions)]
            eprintln!("Running {cmd:?}");

            let mut spawn = cmd.spawn()?;
            if let Some(stdout) = spawn.stdout.take() {
                stream_output_to_receiver(stdout, tx.clone(), OutputType::Stdout);
            }
            if let Some(stderr) = spawn.stderr.take() {
                stream_output_to_receiver(stderr, tx.clone(), OutputType::Stderr);
            }
            match spawn.wait() {
                Err(e) => {
                    eprintln!("Error waiting for occ command: {e:?}");
                    tx.send(Ok(CommandOutput {
                        r#type: OutputType::Exit.into(),
                        message: Some(format!("Error waiting for occ command: {e:?}")),
                        exit_code: Some(500)
                    })).await.expect("send error");
                },
                Ok(status) => {
                    tx.send(Ok(CommandOutput {
                        r#type: OutputType::Exit.into(),
                        message: None,
                        exit_code: status.code(),
                    })).await.expect("send error");
                }
            }
            
            #[cfg(debug_assertions)]
            eprintln!("Command completed successfully");

            Ok(Response::new(ReceiverStream::new(rx)))
        }
    }

    fn stream_output_to_receiver<R>(mut stream: R, tx: Sender<Result<CommandOutput, Status>>, output_type: OutputType)
    where
        R: Read + Send + 'static,
    {
        thread::spawn(move || loop {
            let mut buf = [0];
            match stream.read(&mut buf) {
                Err(e) => {
                    eprintln!("Error reading from stdout: {:?}", e);
                    break;
                },
                Ok(count) => {
                    if count == 0 {
                        break;
                    } else if count == 1 {
                        if let Err(e) = tx.blocking_send(Ok(CommandOutput {
                            message: Some(String::from_utf8_lossy(&buf).parse().unwrap()),
                            r#type: output_type.into(),
                            exit_code: None,
                        })) {
                            eprintln!("Error sending occ output: {e:?}");
                            break;
                        }
                    }
                }
            }
        });
    }
}

pub mod client {
    use std::path::PathBuf;
    use std::sync::Arc;
    use hyper_util::rt::TokioIo;
    use tokio::io;
    use tokio::net::UnixStream;
    use tonic::Streaming;
    use tonic::transport::{Channel, Endpoint, Uri};
    use tower::service_fn;
    use nca_error::NcaError;
    use crate::api::{Command, CommandOutput, OutputType};
    use crate::api::occ_client::OccClient;

    pub async fn run_occ_client(socket_path: String, occ_args: Vec<String>) -> Result<(), NcaError> {

        let channel = get_socket_channel(
            PathBuf::from(socket_path),
            "http://occ.nextcloudatomic.local".to_string()
        ).await.unwrap();

        // let channel = Channel::from_shared(format!("http://{}", &addr)).unwrap()
        //     .connect()
        //     .await.unwrap();

        let mut client = OccClient::new(channel);
        let response = client.exec(Command{arguments: occ_args}).await
            .map_err(|e| NcaError::new_io_error(format!("An error occurred while running occ command: {e:?}")))?
            .into_inner();
        
        handle_occ_output(response).await
    }
    
    pub async fn handle_occ_output(mut output: Streaming<CommandOutput>) -> Result<(), NcaError> {

        loop {
            match output.message().await {
                Err(status) => {
                    eprintln!("Received an error from occ service (status {}): {}",
                              status.code(), status.message());
                    return Err(status.into());
                },
                Ok(message) => {
                    match message {
                        None => { return Ok(()); }
                        Some(out) => {
                            if out.r#type == (OutputType::Exit as i32) {
                                if let Some(msg) = out.message {
                                    eprint!("{msg}");
                                }
                                return if let Some(exit_code) = out.exit_code {
                                    if exit_code == 0 {
                                        Ok(())
                                    } else {
                                        Err(NcaError::new_io_error(format!("occ exited with exit code {exit_code}")))
                                    }
                                } else {
                                    Err(NcaError::new_io_error("occ exited but no exit code was returned!"))
                                }
                            } else if out.r#type == (OutputType::Stderr as i32) {
                                if let Some(msg) = out.message {
                                    eprint!("{msg}");
                                }
                            } else if out.r#type == (OutputType::Stdout as i32) {
                                if let Some(msg) = out.message {
                                    print!("{msg}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn get_socket_channel(socket_path: PathBuf, uri: String) -> Result<Channel, NcaError> {

        let socket = Arc::new(socket_path);

        Endpoint::try_from(uri)
            .map_err( NcaError::new_server_config_error)?
            .connect_with_connector(service_fn(move |_: Uri| {
                let socket = Arc::clone(&socket);
                // Connect to a Uds socket
                async move { Ok::<_, io::Error>(TokioIo::new(UnixStream::connect(&*socket).await?)) }
            }))
            .await
            .map_err(NcaError::new_io_error)

        // let channel = Channel::from_static("");

    }
}