use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterConfig {
    pub bootstrap_servers: String,
    pub postgres_connection_str: String,
    pub update_account_topic: String,
    pub session_timeout_ms: String,
    // Filter by account owners in base58
    pub filter_include_owners: Vec<Vec<String>>,
    // Exception list for filter ( public keys from 32 to 44 characters in base58)
    pub filter_exceptions: Vec<Vec<String>>,
}
