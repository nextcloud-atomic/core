use sha2::Sha256;
use hmac::{Hmac, Mac};
use crate::config::types::{LBCookiePolicy, LoadBalancingPolicy, Match, ReverseProxyHandler, ReverseProxyLoadBalancing, ReverseProxyUpstream, Route, RouteHandler, Server};

type HmacSha256 = Hmac<Sha256>;

pub fn create_nextcloud_server_json(extra_address: String, lb_cookie_secret: String) -> (Server, String) {

    let mut hosts = vec!["localhost".to_string(), "nextcloudatomic.local".to_string()];
    if !hosts.contains(&extra_address) {
        hosts.push(extra_address);
    }
    
    (
        Server {
            listen: vec!["0.0.0.0:443".to_string()],
            routes: vec![
                Route {
                    r#match: Some(vec![
                        Match{
                            host: Some(hosts)
                        }
                    ]),
                    handle: vec![
                        // RouteHandler::Headers(HeadersHandler {
                        //     request: None,
                        //     response: Some(HeaderModification {
                        //         set: Some(HttpHeaders::from([
                        //             ("Content-Type".to_string(), vec!["text/html".to_string()])
                        //         ])),
                        //         add: None,
                        //         delete: None
                        //     })
                        // }),
                        RouteHandler::ReverseProxy(ReverseProxyHandler {
                            upstreams: vec![
                                ReverseProxyUpstream {
                                    dial: "127.0.0.1:1080".to_string(),
                                    max_requests: Some(100)
                                },
                            ],
                            load_balancing: None,
                            // load_balancing: Some(ReverseProxyLoadBalancing {
                            //     selection_policy: LoadBalancingPolicy::Cookie(LBCookiePolicy {
                            //         name: "ncatomic-lb-toggle".to_string(),
                            //         secret: lb_cookie_secret.clone(),
                            //         max_age: None,
                            //         fallback: Box::from(LoadBalancingPolicy::First)
                            //     }),
                            //     retries: None,
                            //     try_duration: None,
                            //     try_interval: None,
                            // }),
                        })
                    ]
                }
            ]
        },
        {
            let mut mac = HmacSha256::new_from_slice(lb_cookie_secret.as_bytes())
                .expect("HMAC can take key of any size");
            mac.update(b"127.0.0.1:3000");
            let lb_cookie_value = mac.finalize();
            hex::encode(lb_cookie_value.into_bytes())
        }
    )
}

pub fn create_nca_setup_server_json() -> Server {
    Server {
        listen: vec!["0.0.0.0:443".to_string()],
        routes: vec![Route {
            r#match: None,
            handle: vec![
                // RouteHandler::Headers(HeadersHandler {
                //     request: None,
                //     response: Some(HeaderModification{
                //         set: Some(HttpHeaders::from([("Content-Type".to_string(), vec!["text/html".to_string()])])),
                //         add: None,
                //         delete: None,
                //     })
                // }),
                RouteHandler::ReverseProxy(ReverseProxyHandler {
                    upstreams: vec![ReverseProxyUpstream {
                        dial: "127.0.0.1:3000".to_string(),
                        max_requests: Some(100),
                    }],
                    load_balancing: None,
                })
            ]
        }]
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use crate::{fix_admin_socket_permissions, CaddyClient};
    use crate::config::builders::{create_nca_setup_server_json, create_nextcloud_server_json};

    async fn test_server_page(cfg: String) -> anyhow::Result<String> {

        assert!(fix_admin_socket_permissions().is_ok());
        let socket_path = env::var("CADDY_ADMIN_SOCKET").expect("Missing env variable: SOCKET_PATH");
        println!("Socket path: {}", &socket_path);
        let caddy = CaddyClient::new(&socket_path)
            .unwrap();
        
        {
            let result = caddy.get_config(None).await;
            assert!(result.is_ok(), "Failed to retrieve caddy config: {:?}", result.expect_err("unknown err"));
            println!("core: {}", result.unwrap())
        }
        {
            // let result = caddy.write_config("site2".to_string(), "apps/srv0/servers/http/".to_string()).await;
            let result = caddy.set_caddy_servers(cfg).await;
            assert!(result.is_ok(), "Failed to load caddy config: {:?}", result.expect_err("unknown err"));
            println!("result: {}", result.unwrap());
        }
        assert!(fix_admin_socket_permissions().is_ok());
        {
            let result = caddy.get_config(None).await;
            assert!(result.is_ok(), "Failed to retrieve caddy config: {:?}", result.expect_err("unknown err"));
            result
        }
    }
    
    #[tokio::test]
    async fn test_activate_nextcloud_server() {
        let cfg = serde_json::to_string(&HashMap::from([("test", create_nextcloud_server_json("127.0.0.1".to_string(), "foo".to_string()))])).unwrap();
        let result = test_server_page(cfg).await;
        println!("core: {}", result.unwrap());
    }
    
    #[tokio::test]
    async fn test_activate_nca_setup_server() {
        let cfg = serde_json::to_string(&HashMap::from([("test", create_nca_setup_server_json())])).unwrap();
        let result = test_server_page(cfg).await;
        println!("core: {}", result.unwrap());
    }
}