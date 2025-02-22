#![allow(non_snake_case)]

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use dioxus::prelude::*;
mod config;
mod api_routes;

use {
    axum::{extract::Extension, routing::get, ServiceExt},
    libsystemd::daemon::NotifyState,
    tower_livereload::LiveReloadLayer,
    grpc_journal::{
        journal_stream::JournalLogStreamService,
        api::journal_log_stream_server::JournalLogStreamServer
    },
    nca_system_api::api::sd_notify,
    tower_http::services::ServeDir,
};

use notify;
use notify::Watcher;
use nca_system_api::types::ServiceStatus;

#[tokio::main]
async fn main() {
    let config = config::Config::new();


    #[cfg(feature = "mock")]
    let service_status_route = {
        get(api_routes::mock::service_status)
            .with_state(api_routes::mock::ServiceMockState {
                service_status_request_count: Arc::new(Mutex::new(0)),
                target_states: HashMap::from([
                    ("nextcloud-all-in-one.service".to_string(), ServiceStatus::ACTIVE)
                ]),
            })
    };
    #[cfg(not(feature = "mock"))]
    let service_status_route = {
        get(api_routes::service_status)
    };

    let mut app = tonic::service::Routes::new(tonic_web::enable(
        JournalLogStreamServer::new(
            JournalLogStreamService::new(false, true))
    ))
        .prepare()
        .into_axum_router();

    app = app.route("/api/service/*name", service_status_route)
        .fallback_service(ServeDir::new("public"))
        .layer(Extension(config.clone()));

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
    
    println!("Listening at {}", config.address);
    let listener = tokio::net::TcpListener::bind(config.address).await.unwrap();
    if let Err(e) = sd_notify(&[NotifyState::Ready]) {
        panic!("{e}");
    };
    if let Err(e) = axum::serve(listener, app).await {
        panic!("server error: {}", e);
    }
}
