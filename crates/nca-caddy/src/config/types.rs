use std::collections::HashMap;
use serde::{Serialize, Serializer};
use serde::ser::{SerializeMap, SerializeStruct};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Serialize)]
pub struct Server {
    pub(crate) listen: Vec<String>,
    pub(crate) routes: Vec<Route>
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Serialize)]
pub struct Route {
    pub(crate) r#match: Option<Vec<Match>>,
    pub(crate) handle: Vec<RouteHandler>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Serialize)]
pub struct Match {
    pub(crate) host: Option<Vec<String>>
}

#[derive(Clone, PartialEq)]
pub enum RouteHandler {
    Headers(HeadersHandler),
    ReverseProxy(ReverseProxyHandler)
}

impl Serialize for RouteHandler {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RouteHandler::Headers(h) => h.serialize(serializer),
            RouteHandler::ReverseProxy(h) => h.serialize(serializer),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct HeadersHandler {
    pub(crate) request: Option<HeaderModification>,
    pub(crate) response: Option<HeaderModification>
}

impl Serialize for HeadersHandler {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("HeadersHandler", 3)?;
        state.serialize_field("handler", "headers")?;
        if self.request.is_some() {
            state.serialize_field("request", &self.request)?;
        }
        if self.response.is_some() {
            state.serialize_field("response", &self.response)?;
        }
        state.end()
    }
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Serialize)]
pub struct HeaderModification {
    pub add: Option<HttpHeaders>,
    pub set: Option<HttpHeaders>,
    pub delete: Option<HttpHeaders>,
}
pub type HttpHeaders = HashMap<String, Vec<String>>;

#[derive(Clone, PartialEq)]
pub struct ReverseProxyHandler {
    pub(crate) upstreams: Vec<ReverseProxyUpstream>,
    pub(crate) load_balancing: Option<ReverseProxyLoadBalancing>
}

impl Serialize for ReverseProxyHandler {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("HeadersHandler", 3)?;
        state.serialize_field("handler", "reverse_proxy")?;
        state.serialize_field("upstreams", &self.upstreams)?;
        if self.load_balancing.is_some() {
            state.serialize_field("load_balancing", &self.load_balancing)?;
        }
        state.end()
    }
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Serialize)]
pub struct ReverseProxyUpstream {
    pub dial: String,
    pub max_requests: Option<usize>
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Serialize)]
pub struct ReverseProxyLoadBalancing {
    pub(crate) selection_policy: LoadBalancingPolicy,
    pub(crate) retries: Option<usize>,
    pub(crate) try_duration: Option<usize>,
    pub(crate) try_interval: Option<usize>
}

#[derive(Clone, PartialEq)]
pub enum LoadBalancingPolicy {
    Cookie(LBCookiePolicy),
    First
}

impl Serialize for LoadBalancingPolicy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            LoadBalancingPolicy::Cookie(lb) => lb.serialize(serializer),
            LoadBalancingPolicy::First => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("policy", "first")?;
                map.end()
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct LBCookiePolicy {
    pub(crate) secret: String,
    pub(crate) name: String,
        pub(crate) max_age: Option<usize>,
        pub(crate) fallback: Box<LoadBalancingPolicy>
    }

impl Serialize for LBCookiePolicy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("HeadersHandler", 5)?;
        state.serialize_field("policy", "cookie")?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("secret", &self.secret)?;
        if self.max_age.is_some() {
            state.serialize_field("max_age", &self.max_age)?;
        }
        state.serialize_field("fallback", &self.fallback.as_ref())?;
        state.end()
    }
}