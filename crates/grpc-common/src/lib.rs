#[cfg(feature = "client")]
pub mod client {
    use std::path::PathBuf;
    use std::sync::Arc;
    use hyper_util::rt::TokioIo;
    use tonic::transport::{Channel, Endpoint, Uri};
    use tower::service_fn;
    use tokio::io;
    use tokio::net::UnixStream;
    use nca_error::NcaError;
    pub async fn get_socket_channel(socket_path: PathBuf, uri: String) -> Result<Channel, NcaError> {
        let socket = Arc::new(socket_path);

        Endpoint::try_from(uri)
            .map_err(NcaError::new_server_config_error)?
            .connect_with_connector(service_fn(move |_: Uri| {
                let socket = Arc::clone(&socket);
                // Connect to a Uds socket
                async move { Ok::<_, io::Error>(TokioIo::new(UnixStream::connect(&*socket).await?)) }
            }))
            .await
            .map_err(NcaError::new_io_error)

        // let channel = Channel::from_static("");

    }
    
    pub async fn retrieve_grpc_channel<S, T, U, V, W>(
        env_socket_address: S, 
        env_socket_directory: T,
        fallback_socket_directory: U,
        fallback_socket_name: V,
        fallback_url: W
    ) -> Result<Channel, NcaError> where 
        S: Into<String>,
        T: Into<String>,
        U: Into<String>,
        V: Into<String>,
        W: Into<String>
    {
        let socket_addr = std::env::var(env_socket_address.into())
            .unwrap_or(
                std::env::var(env_socket_directory.into())
                    .unwrap_or(fallback_socket_directory.into())
                    .trim_end_matches("/")
                    .to_string()
                    + format!("/{}", fallback_socket_name.into()).as_str()
            );
        let channel = if socket_addr.starts_with("http") {
            Endpoint::new(socket_addr)
                .map_err(|e| NcaError::new_io_error(format!("Failed to create endpoint from address: {e:?}")))?
                .connect().await
                .map_err(|e| NcaError::new_io_error(format!("Failed to connect to address: {e:?}")))?
        } else {
            get_socket_channel(
                PathBuf::from(socket_addr),
                fallback_url.into())
                .await.map_err(|e| NcaError::new_io_error(format!("Failed to connect to socket: {e:?}")))?
        };
        Ok(channel)
    }
}

#[cfg(feature = "server")]
pub mod server {
    use listenfd::ListenFd;
    use triggered::Listener;
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio::net::{TcpListener, UnixListener};
    use tonic::codegen::tokio_stream::Stream;
    use tonic::codegen::tokio_stream::wrappers::UnixListenerStream;
    use tonic::IntoStreamingRequest;
    use tonic::transport::server::{Connected, Router, TcpIncoming};
    use nca_error::NcaError;

    pub enum SocketSelectionStrategy {
        First,
        ByName(String)
    }

    pub async fn serve_systemd_socket_tonic(strategy: SocketSelectionStrategy, grpc: Router, stop_trigger: Option<Listener>) -> Result<(), NcaError> {
        let mut sd_fds = ListenFd::from_env();
        
        match sd_fds.take_tcp_listener(0) {
            Ok(fd) => match fd {
                    Some(listener) => {
                        let tcp_listener = TcpListener::from_std(listener)
                            .map_err(NcaError::new_io_error)?;

                        let stream = TcpIncoming::from_listener(tcp_listener, true, None)
                            .map_err(|e| NcaError::new_io_error(format!("Failed to create stream from socket: {e:?}")))
                            ?;
                        serve_socket_tonic(stream, grpc, stop_trigger).await
                            .map_err(|e| NcaError::new_io_error(format!("Failed to create stream from socket: {e:?}")))
                    },
                    None => Err(NcaError::new_server_config_error("No systemd socket found in environment".to_string()))
            },
            Err(_) => match sd_fds.take_unix_listener(0).map_err(NcaError::new_io_error)? {
                Some(listener) => {
                    let stream = UnixListenerStream::new(UnixListener::from_std(listener).unwrap());
                    serve_socket_tonic(stream, grpc, stop_trigger).await
                        .map_err(|e| NcaError::new_io_error(format!("Failed to create stream from socket: {e:?}")))
                },
                None => Err(NcaError::new_server_config_error("No systemd socket found in environment".to_string()))
            }
        }
    }


    pub async fn serve_socket_tonic<I, IO, IE>(stream: I, grpc: Router, stop_trigger: Option<Listener>) -> Result<(), tonic::transport::Error>
    where
        I: Stream<Item=Result<IO, IE>>,
        IO: AsyncRead + AsyncWrite + Connected + Unpin + Send + 'static,
        IO::ConnectInfo: Clone + Send + Sync + 'static,
        IE: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        match stop_trigger {
            Some(trigger) => grpc.serve_with_incoming_shutdown(stream, trigger).await?,
            None => grpc.serve_with_incoming(stream).await?
        }

        Ok(())
    }
}