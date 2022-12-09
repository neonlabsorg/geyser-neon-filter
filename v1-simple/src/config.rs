use ahash::AHashSet;
use log::LevelFilter;
use rdkafka::config::RDKafkaLogLevel;
use serde::{Deserialize, Serialize};
use std::{env, str::FromStr};
use strum_macros::EnumString;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, EnumString)]
pub enum LogLevel {
    /// Higher priority then [`Level::Error`](log::Level::Error) from the log
    /// crate.
    Emerg = 0,
    /// Higher priority then [`Level::Error`](log::Level::Error) from the log
    /// crate.
    Alert = 1,
    /// Higher priority then [`Level::Error`](log::Level::Error) from the log
    /// crate.
    Critical = 2,
    /// Equivalent to [`Level::Error`](log::Level::Error) from the log crate.
    Error = 3,
    /// Equivalent to [`Level::Warn`](log::Level::Warn) from the log crate.
    Warning = 4,
    /// Higher priority then [`Level::Info`](log::Level::Info) from the log
    /// crate.
    Notice = 5,
    /// Equivalent to [`Level::Info`](log::Level::Info) from the log crate.
    Info = 6,
    /// Equivalent to [`Level::Debug`](log::Level::Debug) from the log crate.
    Debug = 7,
}

impl From<&LogLevel> for RDKafkaLogLevel {
    fn from(log_level: &LogLevel) -> Self {
        match log_level {
            LogLevel::Emerg => RDKafkaLogLevel::Emerg,
            LogLevel::Alert => RDKafkaLogLevel::Alert,
            LogLevel::Critical => RDKafkaLogLevel::Critical,
            LogLevel::Error => RDKafkaLogLevel::Error,
            LogLevel::Warning => RDKafkaLogLevel::Warning,
            LogLevel::Notice => RDKafkaLogLevel::Notice,
            LogLevel::Info => RDKafkaLogLevel::Info,
            LogLevel::Debug => RDKafkaLogLevel::Debug,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, EnumString)]
pub enum GlobalLogLevel {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl From<&GlobalLogLevel> for LevelFilter {
    fn from(log_level: &GlobalLogLevel) -> Self {
        match log_level {
            GlobalLogLevel::Off => LevelFilter::Off,
            GlobalLogLevel::Error => LevelFilter::Error,
            GlobalLogLevel::Warn => LevelFilter::Warn,
            GlobalLogLevel::Info => LevelFilter::Info,
            GlobalLogLevel::Debug => LevelFilter::Debug,
            GlobalLogLevel::Trace => LevelFilter::Trace,
        }
    }
}

pub fn env_build_config() -> FilterConfig {
    let filter_log_path = env::var("FILTER_LOG_PATH").expect("FILTER_LOG_PATH is not set");
    let bootstrap_servers = env::var("BOOTSTRAP_SERVERS").expect("BOOTSTRAP_SERVERS is not set");
    let kafka_consumer_group_id =
        env::var("KAFKA_CONSUMER_GROUP_ID").expect("KAFKA_CONSUMER_GROUP_ID is not set");
    let postgres_connection_str =
        env::var("POSTGRES_CONNECTION_STR").expect("POSTGRES_CONNECTION_STR is not set");
    let sasl_username = env::var("SASL_USERNAME").expect("SASL_USERNAME is not set");
    let sasl_password = env::var("SASL_PASSWORD").expect("SASL_PASSWORD is not set");
    let sasl_mechanism = env::var("SASL_MECHANISM").expect("SASL_MECHANISM is not set");
    let security_protocol = env::var("SECURITY_PROTOCOL").expect("SECURITY_PROTOCOL is not set");
    let update_account_topic =
        env::var("UPDATE_ACCOUNT_TOPIC").expect("UPDATE_ACCOUNT_TOPIC is not set");
    let notify_block_topic = env::var("NOTIFY_BLOCK_TOPIC").expect("NOTIFY_BLOCK_TOPIC is not set");
    let update_slot_topic = env::var("UPDATE_SLOT_TOPIC").expect("UPDATE_SLOT_TOPIC is not set");
    let session_timeout_ms = env::var("SESSION_TIMEOUT_MS").expect("SESSION_TIMEOUT_MS is not set");
    let fetch_message_max_bytes =
        env::var("FETCH_MESSAGE_MAX_BYTES").expect("FETCH_MESSAGE_MAX_BYTES is not set");

    let filter_include_owners: AHashSet<String> = env::var("FILTER_INCLUDE_OWNERS")
        .expect("FILTER_INCLUDE_OWNERS is not set")
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let filter_include_pubkeys: AHashSet<String> = env::var("FILTER_INCLUDE_PUBKEYS")
        .expect("FILTER_INCLUDE_PUBKEYS is not set")
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let kafka_log_level: LogLevel =
        LogLevel::from_str(&env::var("KAFKA_LOG_LEVEL").expect("KAFKA_LOG_LEVEL is not set"))
            .unwrap_or(LogLevel::Info);

    let global_log_level: GlobalLogLevel = GlobalLogLevel::from_str(
        &env::var("GLOBAL_LOG_LEVEL").expect("GLOBAL_LOG_LEVEL is not set"),
    )
    .unwrap_or(GlobalLogLevel::Info);

    FilterConfig {
        filter_log_path,
        bootstrap_servers,
        kafka_consumer_group_id,
        postgres_connection_str,
        sasl_username,
        sasl_password,
        sasl_mechanism,
        security_protocol,
        update_account_topic: Some(update_account_topic),
        update_slot_topic: Some(update_slot_topic),
        notify_block_topic: Some(notify_block_topic),
        session_timeout_ms,
        fetch_message_max_bytes,
        filter_include_owners,
        filter_include_pubkeys,
        kafka_log_level,
        global_log_level,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterConfig {
    pub filter_log_path: String,
    pub bootstrap_servers: String,
    pub kafka_consumer_group_id: String,
    pub postgres_connection_str: String,
    pub sasl_username: String,
    pub sasl_password: String,
    pub sasl_mechanism: String,
    pub security_protocol: String,
    pub update_account_topic: Option<String>,
    pub update_slot_topic: Option<String>,
    pub notify_block_topic: Option<String>,
    pub session_timeout_ms: String,
    pub fetch_message_max_bytes: String,
    // Filter by account owners in base58
    pub filter_include_owners: AHashSet<String>,
    // Alway include list for filter ( public keys from 32 to 44 characters in base58 )
    pub filter_include_pubkeys: AHashSet<String>,
    pub kafka_log_level: LogLevel,
    pub global_log_level: GlobalLogLevel,
}
