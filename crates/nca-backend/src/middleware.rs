use std::path::PathBuf;
use axum::body::Body;
use axum::Extension;
use axum::middleware::Next;
use axum::response::IntoResponse;
use http::{Request, StatusCode};
use crate::config::Config;

pub async fn require_setup_not_complete(Extension(config): Extension<Config>, req: Request<Body>, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    #[cfg(feature = "mock-fs")]
    return Ok(next.run(req).await);
    
    if PathBuf::from(config.config_path.as_str()).join("system/setup_complete").exists() {
        #[cfg(debug_assertions)]
        eprintln!("Refusing to serve endpoint: setup already completed");
        Err((StatusCode::PRECONDITION_FAILED, "setupalready completed".to_string()))
    } else {
        Ok(next.run(req).await)
    }
}