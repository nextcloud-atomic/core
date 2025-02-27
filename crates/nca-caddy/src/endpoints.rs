use crate::CaddyClient;

#[derive(Clone, PartialOrd, PartialEq)]
pub enum NcaEndpoint {
    Setup,
    // UNLOCK,
    Maintenance,
    Nextcloud,
    Admin
}

const ALL_ENDPOINTS: &[NcaEndpoint] = &[NcaEndpoint::Setup, NcaEndpoint::Nextcloud, NcaEndpoint::Admin, NcaEndpoint::Maintenance];

pub struct EndpointConfiguration {
    enabled_endpoints: Vec<NcaEndpoint>,
    default_endpoint: NcaEndpoint,
}

struct EndpointCaddyConfig {
    server_name: NcaEndpoint,
    json: &'static str,
}

const SETUP_CONFIG: EndpointCaddyConfig = EndpointCaddyConfig {
    server_name: NcaEndpoint::Setup,
    json: include_str!("../resource/ncatomic_setup.json"),
};
const NEXTCLOUD_CONFIG: EndpointCaddyConfig = EndpointCaddyConfig {
    server_name: NcaEndpoint::Nextcloud,
    json: include_str!("../resource/nextcloud-all-in-one.json"),
};

pub fn configure(caddy: CaddyClient, config: EndpointConfiguration) {
    for endpoint in ALL_ENDPOINTS {
        if config.enabled_endpoints.contains(endpoint) {
            
        }
    }
}
