pub mod layout;
pub mod assets;
pub mod components;

use bytes::Bytes;
use http::{StatusCode};
use paspio::entropy;
use rand::Rng;
use reqwest::{Body, Response, Url};
use serde::de::DeserializeOwned;
pub use crate::components::*;

// use web_sys::window;

#[cfg(not(feature = "mock-backend"))]
pub fn base_url() -> String {
    window().unwrap().location().origin().unwrap()
}
#[cfg(feature = "mock-backend")]
pub fn base_url() -> String {
    "http://localhost:3000".to_string()
}

#[derive(Debug)]
pub(crate) struct HttpResponse {
    _inner: HttpResponseWrapper
}

// const DEFAULT_HTTP_HEADERS: HeaderMap = HeaderMap::default();

impl HttpResponse {

    pub fn status(&self) -> StatusCode {
        match &self._inner {
            HttpResponseWrapper::Reqwest(r) => r.status(),
            HttpResponseWrapper::Mocked(r) => r.status
        }
    }

    // pub fn version(&self) -> Version {
    //     match &self._inner {
    //         HttpResponseWrapper::Reqwest(r) => r.version(),
    //         HttpResponseWrapper::Mocked(r) => Version::default()
    //     }
    // }

    // pub fn headers(&self) -> HeaderMap {
    //     match &self._inner {
    //         HttpResponseWrapper::Reqwest(r) => r.headers().clone(),
    //         HttpResponseWrapper::Mocked(r) => DEFAULT_HTTP_HEADERS
    //     }
    // }

    pub fn content_length(self) -> Result<Option<u64>, String> {
        match self._inner {
            HttpResponseWrapper::Reqwest(r) => Ok(r.content_length()),
            HttpResponseWrapper::Mocked(r) => Ok(Some(r.body.len().try_into()
                .map_err(|_| "Failed to convert usize to u64".to_string())?))
        }
    }

    pub fn url(&self) -> &Url {
        match &self._inner {
            HttpResponseWrapper::Reqwest(r) => r.url(),
            HttpResponseWrapper::Mocked(r) => &r.url
        }
    }

    // pub fn remote_addr(self) -> Option<SocketAddr> {
    //     match self._inner {
    //         HttpResponseWrapper::Reqwest(r) => r.remote_addr(),
    //         HttpResponseWrapper::Mocked(_) => None
    //     }
    // }

    // pub fn extensions(&self) -> &Extensions {
    //     match self._inner.borrow() {
    //         HttpResponseWrapper::Reqwest(r) => r.extensions(),
    //         HttpResponseWrapper::Mocked(r) => &Extensions::default()
    //     }
    // }

    pub async fn bytes(self) -> Result<bytes::Bytes, String> {
        
        match self._inner {
            HttpResponseWrapper::Reqwest(r) => r.bytes().await.map_err(|e| e.to_string()),
            HttpResponseWrapper::Mocked(r) => {
                let body = r.body.clone();
                let result = body.as_bytes().to_owned();
                Ok(Bytes::from(result))
            } 
        }
    }
    
    pub async fn text(self) -> Result<String, String> {
        match self._inner {
            HttpResponseWrapper::Reqwest(r) => r.text().await.map_err(|e| e.to_string()),
            HttpResponseWrapper::Mocked(r) => Ok(r.body)
        }
    }
    
    pub async fn json<T: DeserializeOwned>(self) -> Result<T, String> {
        match self._inner {
            HttpResponseWrapper::Reqwest(r) => r.json().await.map_err(|e| e.to_string()),
            HttpResponseWrapper::Mocked(r) => serde_json::from_slice(r.body.as_bytes()).map_err(|e| e.to_string())
        }
    }
    
    // pub async fn chunk(&mut self) -> Result<Option<bytes::Bytes>, String> {
    //     match &mut self._inner {
    //         HttpResponseWrapper::Reqwest(r) => r.chunk().await.map_err(|e| e.to_string()),
    //         HttpResponseWrapper::Mocked(r) => Ok(Some(bytes::Bytes::from(r.body.clone())))
    //     }
    // }
    
}

impl From<reqwest::Response> for HttpResponse {
    fn from(value: Response) -> Self {
        HttpResponse {
            _inner: HttpResponseWrapper::Reqwest(value)
        }
    }
}

impl From<MockResponse> for HttpResponse {
    fn from(value: MockResponse) -> Self {
        HttpResponse {
            _inner: HttpResponseWrapper::Mocked(value)
        }
    }
}

#[derive(Debug)]
enum HttpResponseWrapper {
    Reqwest(reqwest::Response),
    Mocked(MockResponse)
}

#[derive(Debug)]
struct MockResponse {
    status: StatusCode,
    body: String,
    url: Url
}

pub(crate) async fn do_post<T: Into<Body>>(request_url: &str, body: T, content_type: Option<&str>) -> Result<HttpResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .post(request_url)
        .header("Content-Type", content_type.unwrap_or("application/json"))
        .body(body.into())
        .send()
        .await
        .map(|r| r.into())
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ConfigStep {
    Welcome,
    ConfigurePasswords,
    ConfigureNextcloud,
    ConfigureDisks,
    Startup
}

impl ConfigStep {
    
    pub fn hasNext(self) -> bool {
        self != ConfigStep::Startup
    }
    pub fn next(self) -> Option<ConfigStep> {
        match self {
            ConfigStep::Welcome => Some(ConfigStep::ConfigurePasswords),
            ConfigStep::ConfigurePasswords => Some(ConfigStep::ConfigureNextcloud),
            ConfigStep::ConfigureNextcloud => Some(ConfigStep::ConfigureDisks),
            ConfigStep::ConfigureDisks => Some(ConfigStep::Startup),
            ConfigStep::Startup => None,
        }

    }
    
    pub fn hasPrevious(self) -> bool {
        self != ConfigStep::Welcome
    }
    
    pub fn previous(self) -> Option<ConfigStep> {
        match self {
            ConfigStep::Startup => Some(ConfigStep::ConfigureDisks),
            ConfigStep::ConfigureDisks => Some(ConfigStep::ConfigureNextcloud),
            ConfigStep::ConfigureNextcloud => Some(ConfigStep::ConfigurePasswords),
            ConfigStep::ConfigurePasswords => Some(ConfigStep::Welcome),
            ConfigStep::Welcome => None,
        }
    }
}

pub fn generate_secure_password() -> String {
    rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(24).map(char::from)
        .collect()
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum PasswordStrength {
    Insecure,
    Weak,
    Strong
}

pub fn check_is_secure_password(pw: String) -> PasswordStrength {
    if pw.is_empty() {
        return PasswordStrength::Insecure;
    }
    match entropy(&pw) {
        e if e < 100.0 => PasswordStrength::Insecure,
        e if e < 130.0 => PasswordStrength::Weak,
        _ =>  PasswordStrength::Strong,
    }
}