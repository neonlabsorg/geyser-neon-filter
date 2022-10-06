mod config;
mod consumer;
mod filter;

use crate::consumer::consumer;
use config::FilterConfig;
use fast_log::{
    consts::LogSize,
    plugin::{file_split::RollingType, packer::LogPacker},
    Config, Logger,
};
use filter::filter;
use log::error;
use tokio::fs;

#[tokio::main]
async fn main() {
    let _logger: &'static Logger = fast_log::init(Config::new().console().file_split(
        "/var/logs/neon_filter.log",
        LogSize::KB(512),
        RollingType::All,
        LogPacker {},
    ))
    .expect("Failed to initialize fast_log");

    let contents = fs::read_to_string("filter_config.json")
        .await
        .expect("Failed to read filter_config.json");

    let result: serde_json::Result<FilterConfig> = serde_json::from_str(&contents);
    match result {
        Ok(config) => {
            let (filter_tx, filter_rx) = flume::unbounded();
            let filter_loop_handle = tokio::spawn(filter(config.clone(), filter_rx));
            let consumer_loop_handle = tokio::spawn(consumer(config, filter_tx));
            let _ = consumer_loop_handle.await;
            let _ = filter_loop_handle.await;
        }
        Err(e) => error!("Failed to parse filter config, error {e}"),
    }
}
