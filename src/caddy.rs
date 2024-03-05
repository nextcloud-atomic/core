use std::io::{BufWriter, Write};
use std::str::from_utf8;
use http_body_util::{Full, BodyExt};
use hyper::body::Bytes;
use hyper::{Method, Request};
use crate::error::NcpError;
use hyperlocal::{UnixConnector, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;


pub struct CaddyClient {
    socket_path: String,
    client: Client<UnixConnector, Full<Bytes>>
}

impl CaddyClient {

    pub fn new(socket_path: &str) -> Result<CaddyClient, NcpError> {
        Ok(CaddyClient{
            socket_path: socket_path.to_string(),
            client: Client::new()
        })
    }
    // pub async fn load_config(&self, caddy_config: String, config_path: Option<String>) -> Result<String, NcpError>{
    //     let uri = Uri::new(&self.socket_path, &("/load/".to_owned() + &config_path.unwrap_or("".to_string())));
    //     let req = Request::builder()
    //         .uri(uri)
    //         .body(Full::from(caddy_config))?;
    //     let mut response = self.client.request(req).await?;
    //     if !response.status().is_success() {
    //         return Err(NcpError::from(format!(
    //             "Failed to load caddy config (received status: {})",
    //             response.status().as_str())))
    //     }
    //     let mut buf = BufWriter::new(Vec::new());
    //     while let Some(frame_result) = response.frame().await {
    //         let frame = frame_result?;
    //         if let Some(segment) = frame.data_ref() {
    //             buf.write_all(segment.iter().as_slice())?;
    //         }
    //     }
    //     let result = String::from_utf8(buf.into_inner()?)?;
    //     Ok(result)
    // }
    pub async fn load_config(&self, caddy_config: String, config_path: Option<String>) -> Result<String, NcpError> {
        let stream = TokioIo::new(UnixStream::connect(self.socket_path.as_str()).await?);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(stream).await
            .map_err(|e| e.to_string())?;
        conn.await.map_err(|e| e.to_string())?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });
        let request = Request::builder()
            .method(Method::POST)
            .uri("http://127.0.0.1/load/".to_owned() + config_path.unwrap_or("".into()).as_str())
            .body(Full::from(caddy_config)).map_err(|e| e.to_string())?;
        let response = sender.send_request(request).await.map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err(NcpError::from(format!("Failed to load caddy config (status {})", response.status().to_string())));
        }
        let body = response.collect().await.map_err(|e| e.to_string())?.to_bytes();
        Ok(String::from_utf8(body.to_vec()).map_err(|e| e.to_string())?)
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use crate::caddy::CaddyClient;
    use super::*;

    #[test]
    fn load_caddy_config() {
        let caddy = CaddyClient::new("/var/run/caddy/admin.sock")
            .unwrap();
        let mut f = File::options().read(true).open("resource/caddy/default_nc_aio.json")?;
        let mut cfg = String::new();
        f.read_to_string(&mut cfg)?;
        if let Some(response) = caddy.load_config(cfg, None) {
            println!("{}", response)
        }
    }
}
