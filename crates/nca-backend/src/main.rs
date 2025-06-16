#![allow(non_snake_case)]

use std::collections::HashMap;
#[cfg(feature = "watch")]
use {
    std::path::Path,
    tower_livereload::LiveReloadLayer,
};
#[cfg(feature = "mock-systemd")]
use {
    std::sync::{Arc, Mutex},
    nca_system_api::systemd::types::ServiceStatus
};
use axum::Router;
use axum::routing::post;
use dioxus::prelude::*;

mod config;
mod api_routes;
mod middleware;
mod crypto;

use {
    axum::{extract::Extension, routing::get, ServiceExt},
    libsystemd::daemon::NotifyState,
    grpc_journal::{
        journal_stream::JournalLogStreamService,
        api::journal_log_stream_server::JournalLogStreamServer
    },
    nca_system_api::systemd::api::sd_notify,
    tower_http::services::ServeDir,
};

use notify::Watcher;
use nca_caddy::CaddyClient;
use nca_caddy::config::builders::create_nca_setup_server_json;
use crate::api_routes::{activate_endpoint_nextcloud, configure_nextcloud_atomic, generate_credentials};
use crate::middleware::require_setup_not_complete;

#[tokio::main]
async fn main() {
    let config = config::Config::new();


    #[cfg(feature = "mock-systemd")]
    let service_status_route = {
        get(api_routes::mock::service_status)
            .with_state(api_routes::mock::ServiceMockState {
                service_status_request_count: Arc::new(Mutex::new(0)),
                target_states: HashMap::from([
                    ("nextcloud-all-in-one.service".to_string(), ServiceStatus::ACTIVE)
                ]),
            })
    };

    #[cfg(not(feature = "mock-systemd"))]
    let service_status_route = {
        get(api_routes::service_status)
    };

    let mut setup_router = Router::new()
        .route("/configure", post(configure_nextcloud_atomic))
        .route("/credentials", post(generate_credentials))
        .route("/caddy/endpoint/enable/nextcloud", post(activate_endpoint_nextcloud))
        .route("/service/*name", service_status_route);

    let mut app = tonic::service::Routes::new(tonic_web::enable(
        JournalLogStreamServer::new(
            JournalLogStreamService::new(false, true))
    ))
        .prepare()
        .into_axum_router();

    #[cfg(feature = "insecure")]
    {
        app = app.layer(tower_http::cors::CorsLayer::permissive());
        setup_router = setup_router.layer(tower_http::cors::CorsLayer::permissive());
    }

    app = app
        .route_layer(axum::middleware::from_fn(require_setup_not_complete))
        .nest_service("/api/setup", setup_router)
        .fallback_service(ServeDir::new("public"))
        .layer(Extension(config.clone()));

    #[cfg(feature = "insecure")]
    {
        app = app.layer(tower_http::cors::CorsLayer::permissive());
    }

    #[cfg(feature = "watch")]
    {
        let livereload = LiveReloadLayer::new();
        let reloader = livereload.reloader();
        app = app.layer(livereload);

        let mut watcher = notify::recommended_watcher(move |_| reloader.reload())
            .expect("Failed to setup file watcher");
        watcher.watch(Path::new("public"), notify::RecursiveMode::Recursive)
            .expect("Failed to watch /public path");
    }
    
    #[cfg(not(feature = "mock-caddy"))]
    if let Some(socket_addr) = config.caddy_admin_socket {
        println!("Setting up caddy...");
        let caddy = CaddyClient::new(&socket_addr)
            .expect("Failed to initialize caddy client");
        let servers_cfg = serde_json::to_string(&HashMap::from([("nca-web", create_nca_setup_server_json())]))
            .expect("Failed to generate caddy server config");
        caddy.set_caddy_servers(servers_cfg).await
            .expect("Failed to configure caddy");
    }
    
    println!("Listening at {}", config.address);
    let listener = tokio::net::TcpListener::bind(config.address).await.unwrap();
    if let Err(e) = sd_notify(&[NotifyState::Ready]) {
        panic!("{e}");
    };
    if let Err(e) = axum::serve(listener, app).await {
        panic!("server error: {}", e);
    }
}
