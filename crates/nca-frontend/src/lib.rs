pub mod layout;
pub mod assets;
pub mod components;

use reqwest::{Body, Response};
pub use crate::components::*;

use web_sys::window;

#[cfg(not(feature = "mock-backend"))]
pub fn base_url() -> String {
    window().unwrap().location().origin().unwrap()
}
#[cfg(feature = "mock-backend")]
pub fn base_url() -> String {
    "http://localhost:3000".to_string()
}

pub(crate) async fn do_post<T: Into<Body>>(request_url: &str, body: T, content_type: Option<&str>) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .post(request_url)
        .header("Content-Type", content_type.unwrap_or("application/json"))
        .body(body.into())
        .send()
        .await
}
