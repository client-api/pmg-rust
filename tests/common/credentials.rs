use clientapi_pmg::apis::configuration::{ApiKey, Configuration};
use reqwest::header::{HeaderMap, HeaderValue};

/// PMG has no token API on 9.x — `token_value` will be the sentinel
/// `"(unsupported-by-pmg)"`. Tests must check `token_auth_supported()`
/// before invoking any token-related path (they should not in this suite,
/// but the helper is kept for symmetry with the PVE / PBS / PDM repos).
#[derive(Debug, Clone)]
pub struct Credentials {
    pub url: String,
    pub user: String,
    pub password: String,
    pub token_header_value: String,
    pub token_value: String,
}

impl Credentials {
    pub fn from_env() -> Self {
        Self {
            url: env_required("PROXMOX_URL"),
            user: env_required("PROXMOX_USER"),
            password: env_required("PROXMOX_PASSWORD"),
            token_header_value: env_required("PROXMOX_TOKEN_HEADER_VALUE"),
            token_value: env_required("PROXMOX_TOKEN_VALUE"),
        }
    }

    pub fn token_auth_supported(&self) -> bool {
        self.token_value != "(unsupported-by-pmg)"
    }

    fn base_client() -> reqwest::Client {
        reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("reqwest client")
    }

    pub fn config_anonymous(&self) -> Configuration {
        let mut cfg = Configuration::new();
        cfg.base_path = self.url.trim_end_matches('/').to_string() + "/api2/json";
        cfg.client = Self::base_client();
        cfg.api_key = None;
        cfg
    }

    /// Token-authenticated config is exposed for parity with the other
    /// products, but PMG-side calls using it will return 401 — tests must
    /// guard with `skip_if_pmg!()`.
    pub fn config_with_token(&self) -> Configuration {
        let mut cfg = self.config_anonymous();
        cfg.api_key = Some(ApiKey {
            prefix: None,
            key: self.token_header_value.clone(),
        });
        cfg
    }

    pub fn config_with_ticket(&self, ticket: &str, csrf: &str) -> Configuration {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            HeaderValue::from_str(&format!("PMGAuthCookie={ticket}"))
                .expect("ticket header value"),
        );
        headers.insert(
            "CSRFPreventionToken",
            HeaderValue::from_str(csrf).expect("csrf header value"),
        );
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .default_headers(headers)
            .build()
            .expect("reqwest client");
        let mut cfg = self.config_anonymous();
        cfg.client = client;
        cfg
    }

    pub fn config_with_ticket_no_csrf(&self, ticket: &str) -> Configuration {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            HeaderValue::from_str(&format!("PMGAuthCookie={ticket}"))
                .expect("ticket header value"),
        );
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .default_headers(headers)
            .build()
            .expect("reqwest client");
        let mut cfg = self.config_anonymous();
        cfg.client = client;
        cfg
    }
}

fn env_required(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("env var {key} must be set"))
}
