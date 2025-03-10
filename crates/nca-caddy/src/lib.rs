pub mod config;

use std::fmt::Debug;
use std::process::{Command, ExitStatus};
use http_body_util::{Empty, Full, BodyExt};
use hyper::body::Bytes;
use hyper::{Method, Request};
use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;
use anyhow::{bail, Result};


pub struct CaddyClient {
    socket_path: String,
    // client: Client<UnixConnector, Full<Bytes>>
}

impl CaddyClient {

    pub fn new(socket_path: &str) -> Result<CaddyClient> {
        Ok(CaddyClient{
            socket_path: socket_path.to_string(),
            // client: Client::new()
        })
    }
    #[cfg(feature = "mock")]
    pub async fn load_config(&self, caddy_config: String, config_path: Option<String>) -> Result<String> {
        Ok("{}".to_string())
    }
    
    #[cfg(not(feature = "mock"))]
    pub async fn load_config(&self, caddy_config: String, config_path: Option<String>) -> Result<String> {
        let stream = TokioIo::new(UnixStream::connect(self.socket_path.as_str()).await?);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(stream).await?;
        //conn.await.map_err(|e| e.to_string())?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });
        let request: Request<Full<Bytes>> = Request::builder()
            .method(Method::POST)
            .header("Host", "127.0.0.1")
            .header("Content-Type", "application/json")
            .uri("http://localhost/config/apps".to_owned() + match config_path {
                Some(val) => match val.starts_with("/") {
                    true => val.to_string(),
                    false => "/".to_string() + val.as_str(),
                },
                None => "".to_string()
            }.as_str())
            .body(Full::from(caddy_config))?;
        println!("request: {:?}", request);
        let response = sender.send_request(request).await?;
        let status = response.status();
        let body = response.collect().await?.to_bytes();
        if !status.is_success() {
            bail!("Failed to load caddy core (status {}): {}", status.to_string(), String::from_utf8_lossy(&body));
        }
        Ok(String::from_utf8(body.to_vec())?)
    }
    
    #[cfg(feature = "mock")]
    pub async fn change_config(&self, method: Method, payload: Option<String>, config_path: Option<String>) -> Result<String> {
        Ok("{}".to_string())
    }

    #[cfg(not(feature = "mock"))]
    pub async fn change_config(&self, method: Method, payload: Option<String>, config_path: String) -> Result<String> {
        let stream = TokioIo::new(UnixStream::connect(self.socket_path.as_str()).await?);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(stream).await?;
        //conn.await.map_err(|e| e.to_string())?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });
        let request_body = match payload {
            Some(s) => Full::from(s),
            None => Full::default()
        };
        let request: Request<Full<Bytes>> = Request::builder()
            .method(method)
            .header("Host", "127.0.0.1")
            .header("Content-Type", "application/json")
            .uri("http://localhost/config".to_owned() + match config_path.starts_with("/") {
                true => config_path.to_string(),
                false => "/".to_string() + config_path.as_str()
            }.as_str())
            .body(request_body)?;
        println!("request: {:?}", request);
        let response = sender.send_request(request).await?;
        println!("response: {response:?}");
        let status = response.status();
        let body = response.collect().await?.to_bytes();
        if !status.is_success() {
            return bail!("Failed to write caddy core (status {}): {}", status.to_string(), String::from_utf8_lossy(&body));
        }
        Ok(String::from_utf8(body.to_vec())?)
    }
    
    #[cfg(feature = "mock")]
    pub async fn get_config(&self, config_path: Option<String>) -> Result<String> {
        Ok("{}".to_string())
    }
    
    #[cfg(not(feature = "mock"))]
    pub async fn get_config(&self, config_path: Option<String>) -> Result<String> {
        let stream = TokioIo::new(UnixStream::connect(self.socket_path.as_str()).await?);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(stream).await?;
        //conn.await.map_err(|e| e.to_string())?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });
        let request: Request<Empty<Bytes>> = Request::builder()
            .method(Method::GET)
            .header("Host", "127.0.0.1")
            .uri("http://localhost/config/".to_owned() + config_path.unwrap_or("".into()).as_str())
            .body(Empty::new())?;
        let response = sender.send_request(request).await?;
        let status = response.status();
        let body = response.collect().await?.to_bytes().clone();
        if !status.is_success() {
            bail!("Failed to retrieve caddy core (status {}): {}", status.to_string(), String::from_utf8_lossy(&body));
        }
        Ok(String::from_utf8(body.to_vec())?)
    }

    #[cfg(feature = "mock")]
    pub async fn set_caddy_servers(&self, servers_cfg: String) -> Result<String> {
        Ok("{}".to_string())
    }
    
    #[cfg(not(feature = "mock"))]
    pub async fn set_caddy_servers(&self, servers_cfg: String) -> Result<String>{
        self.change_config(Method::POST, Some(servers_cfg), "/apps/http/servers".to_string()).await
    }

    #[cfg(feature = "mock")]
    pub async fn set_server_static_response(&self, server_name: String, html_body: String) -> Result<String> {
        Ok("{}".to_string())
    }
    
    #[cfg(not(feature = "mock"))]
    pub async fn set_server_static_response(&self, server_name: String, html_body: String) -> Result<String>{
        let payload = format!(r#"[
            {{
                "handle": [
                    {{
                        "handler": "static_response",
                        "body": "{html_body}"
                    }}
                ]
            }}
        ]"#);
        self.set_server_route(server_name, payload).await
    }

    #[cfg(feature = "mock")]
    pub async fn set_server_route(&self, server_name: String, route_config: String) -> Result<String> {
        Ok("{}".to_string())
    }
    
    #[cfg(not(feature = "mock"))]
    pub async fn set_server_route(&self, server_name: String, route_config: String) -> Result<String> {
        self.change_config(Method::DELETE, None, format!("/apps/http/servers/{server_name}/routes")).await?;
        #[cfg(test)]
        fix_admin_socket_permissions().expect("Failed to fix socket permissions");
        self.change_config(Method::POST, Some(route_config), format!("/apps/http/servers/{server_name}/routes")).await
    }
}

#[cfg(test)]
fn fix_admin_socket_permissions() -> Result<(), ExitStatus>{
    let status = Command::new("docker")
        .args(&["exec", "caddy", "chmod", "g+rwx", "/run/caddy/admin.sock"])
        .status()
        .expect("Failed to fix ownership of admin socket");
    match status.success() {
        true => Ok(()),
        false => Err(status)
    }
}


#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::Read;
    use hyper::Method;
    use crate::CaddyClient;
    use super::fix_admin_socket_permissions;

    #[tokio::test]
    async fn test_change_caddy_config() {
        assert!(fix_admin_socket_permissions().is_ok());
        let socket_path = env::var("CADDY_ADMIN_SOCKET").expect("Missing env variable: SOCKET_PATH");
        println!("Socket path: {}", &socket_path);
        let caddy = CaddyClient::new(&socket_path)
            .unwrap();
        let mut f = File::options().read(true).open("resource/test_page.json").unwrap();
        let mut cfg = String::new();
        f.read_to_string(&mut cfg).unwrap();
        {
            let result = caddy.get_config(None).await;
            assert!(result.is_ok(), "Failed to retrieve caddy config: {:?}", result.expect_err("unknown err"));
            println!("core: {}", result.unwrap())
        }
        {
            // let result = caddy.write_config("site2".to_string(), "apps/srv0/servers/http/".to_string()).await;
            let result = caddy.change_config(Method::POST, Some(cfg), "/apps".to_string()).await;
            assert!(result.is_ok(), "Failed to load caddy config: {:?}", result.expect_err("unknown err"));
            println!("result: {}", result.unwrap());
        }
        assert!(fix_admin_socket_permissions().is_ok());
        {
            let result = caddy.get_config(None).await;
            assert!(result.is_ok(), "Failed to retrieve caddy config: {:?}", result.expect_err("unknown err"));
            println!("core: {}", result.unwrap())
        }
    }

    #[tokio::test]
    async fn test_set_static_response() {
        assert!(fix_admin_socket_permissions().is_ok());
        let socket_path = env::var("CADDY_ADMIN_SOCKET").expect("Missing env variable: SOCKET_PATH");
        println!("Socket path: {}", &socket_path);
        let caddy = CaddyClient::new(&socket_path)
            .unwrap();
        let mut f = File::options().read(true).open("resource/test_page.json").unwrap();
        let mut cfg = String::new();
        f.read_to_string(&mut cfg).unwrap();
        {
            // let result = caddy.write_config("site2".to_string(), "apps/srv0/servers/http/".to_string()).await;
            let result = caddy.load_config(cfg, None).await;
            assert!(result.is_ok(), "Failed to load caddy config: {:?}", result.expect_err("unknown err"));
            println!("result: {}", result.unwrap());
        }
        assert!(fix_admin_socket_permissions().is_ok());
        {
            let result = caddy.get_config(None).await;
            assert!(result.is_ok(), "Failed to retrieve caddy config: {:?}", result.expect_err("unknown err"));
            println!("core: {}", result.unwrap())
        }
        let static_html = "hello world";
        {
            let result = caddy.set_server_static_response("test".to_string(), static_html.to_string()).await;
            assert!(result.is_ok(), "Failed to set static response for server 'test': {}", result.unwrap_err());
        }
        assert!(fix_admin_socket_permissions().is_ok());
        {
            let result = caddy.get_config(None).await;
            assert!(result.is_ok(), "Failed to retrieve caddy config: {:?}", result.expect_err("unknown err"));
            println!("core: {}", result.unwrap())
        }
        let http_result = reqwest::get("http://localhost").await;
        assert!(http_result.is_ok(), "Failed to retrieve page from caddy: {}", http_result.unwrap_err());
        assert_eq!(http_result.unwrap().text().await.unwrap(), static_html)

    }
}
