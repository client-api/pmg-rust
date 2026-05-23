use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ResponseContent<T> {
    pub status: reqwest::StatusCode,
    pub content: String,
    pub entity: Option<T>,
}

#[derive(Debug)]
pub enum Error<T> {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
    ResponseError(ResponseContent<T>),
}

impl <T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (module, e) = match self {
            Error::Reqwest(e) => ("reqwest", e.to_string()),
            Error::Serde(e) => ("serde", e.to_string()),
            Error::Io(e) => ("IO", e.to_string()),
            Error::ResponseError(e) => ("response", format!("status code {}", e.status)),
        };
        write!(f, "error in {}: {}", module, e)
    }
}

impl <T: fmt::Debug> error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match self {
            Error::Reqwest(e) => e,
            Error::Serde(e) => e,
            Error::Io(e) => e,
            Error::ResponseError(_) => return None,
        })
    }
}

impl <T> From<reqwest::Error> for Error<T> {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl <T> From<serde_json::Error> for Error<T> {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

impl <T> From<std::io::Error> for Error<T> {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub fn urlencode<T: AsRef<str>>(s: T) -> String {
    ::url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

pub fn parse_deep_object(prefix: &str, value: &serde_json::Value) -> Vec<(String, String)> {
    if let serde_json::Value::Object(object) = value {
        let mut params = vec![];

        for (key, value) in object {
            match value {
                serde_json::Value::Object(_) => params.append(&mut parse_deep_object(
                    &format!("{}[{}]", prefix, key),
                    value,
                )),
                serde_json::Value::Array(array) => {
                    for (i, value) in array.iter().enumerate() {
                        params.append(&mut parse_deep_object(
                            &format!("{}[{}][{}]", prefix, key, i),
                            value,
                        ));
                    }
                },
                serde_json::Value::String(s) => params.push((format!("{}[{}]", prefix, key), s.clone())),
                _ => params.push((format!("{}[{}]", prefix, key), value.to_string())),
            }
        }

        return params;
    }

    unimplemented!("Only objects are supported with style=deepObject")
}

/// Internal use only
/// A content type supported by this client.
#[allow(dead_code)]
enum ContentType {
    Json,
    Text,
    Unsupported(String)
}

impl From<&str> for ContentType {
    fn from(content_type: &str) -> Self {
        if content_type.starts_with("application") && content_type.contains("json") {
            return Self::Json;
        } else if content_type.starts_with("text/plain") {
            return Self::Text;
        } else {
            return Self::Unsupported(content_type.to_string());
        }
    }
}

pub mod access_api;
pub mod access_auth_realm_api;
pub mod access_oidc_api;
pub mod access_tfa_api;
pub mod access_ticket_api;
pub mod access_users_api;
pub mod config_api;
pub mod config_acme_api;
pub mod config_admin_api;
pub mod config_clamav_api;
pub mod config_cluster_api;
pub mod config_customscores_api;
pub mod config_dkim_api;
pub mod config_domains_api;
pub mod config_fetchmail_api;
pub mod config_ldap_api;
pub mod config_mail_api;
pub mod config_mynetworks_api;
pub mod config_pbs_api;
pub mod config_ruledb_api;
pub mod config_spam_api;
pub mod config_spamquar_api;
pub mod config_tfa_api;
pub mod config_tls_inbound_domains_api;
pub mod config_tlspolicy_api;
pub mod config_transport_api;
pub mod config_virusquar_api;
pub mod config_welcomelist_api;
pub mod config_whitelist_api;
pub mod nodes_api;
pub mod nodes_apt_api;
pub mod nodes_backup_api;
pub mod nodes_certificates_api;
pub mod nodes_clamav_api;
pub mod nodes_config_api;
pub mod nodes_dns_api;
pub mod nodes_network_api;
pub mod nodes_pbs_api;
pub mod nodes_postfix_api;
pub mod nodes_services_api;
pub mod nodes_spamassassin_api;
pub mod nodes_status_api;
pub mod nodes_subscription_api;
pub mod nodes_tasks_api;
pub mod nodes_time_api;
pub mod nodes_tracker_api;
pub mod quarantine_api;
pub mod quarantine_blacklist_api;
pub mod quarantine_blocklist_api;
pub mod quarantine_content_api;
pub mod quarantine_welcomelist_api;
pub mod quarantine_whitelist_api;
pub mod statistics_api;
pub mod version_api;

pub mod configuration;
