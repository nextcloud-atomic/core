use std::ffi::OsStr;
use std::net::SocketAddr;
use std::path::PathBuf;
use axum::{extract::Extension, response::Json, routing::get, Router};
use axum::response::Html;
use dioxus::prelude::VirtualDom;
use serde::{Deserialize, Serialize};
use sysinfo::{Disk, DiskKind, Disks, System};
use errors::NcAtomicError;
use model::disk::DiskInfo;
use ui::disks::{IndexPage, IndexPageProps};
use ui::render;

#[tokio::main]
async fn main() {

    // build our application with a route
    let app = Router::new()
        .route("/", get(disks));
        // .layer(Extension(config))
        // .layer(Extension(pool.clone()));

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn disks_rest() -> Result<Json<Vec<DiskInfo>>, NcAtomicError> {
    let disks = Disks::new_with_refreshed_list().iter()
        .map(| d| d.into()).collect::<Vec<DiskInfo>>();
    Ok(Json(disks))
}

async fn disks() -> Result<Html<String>, NcAtomicError> {
    let disks = Disks::new_with_refreshed_list().iter()
        .map(|d| d.into()).collect::<Vec<DiskInfo>>();

    let html = render(VirtualDom::new_with_props(
        IndexPage,
        IndexPageProps { disks },
    ));
    Ok(Html(html))
}