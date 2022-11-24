mod config;
mod consumer;
mod db;
mod filter;

use std::sync::Arc;

use crate::consumer::consumer;
use clap::{Arg, Command};
use config::{env_build_config, FilterConfig};
use crossbeam_queue::SegQueue;
use db::{db_statement_executor, initialize_db_client, DbAccountInfo};
use fast_log::{
    consts::LogSize,
    plugin::{file_split::RollingType, packer::LogPacker},
    Config, Logger,
};
use filter::filter;
use log::error;
use tokio::fs;

async fn run(config: FilterConfig, logger: &'static Logger) {
    let config = Arc::new(config);
    let db_queue: Arc<SegQueue<DbAccountInfo>> = Arc::new(SegQueue::new());

    logger.set_level((&config.global_log_level).into());

    let client = initialize_db_client(config.clone()).await;

    let (filter_tx, filter_rx) = flume::unbounded();
    let filter_loop_handle = tokio::spawn(filter(config.clone(), db_queue.clone(), filter_rx));
    let consumer_loop_handle = tokio::spawn(consumer(config.clone(), filter_tx));
    let db_statement_executor_handle =
        tokio::spawn(db_statement_executor(config, client, db_queue));

    let _ = consumer_loop_handle.await;
    let _ = filter_loop_handle.await;
    let _ = db_statement_executor_handle.await;
}

#[tokio::main]
async fn main() {
    let app = Command::new("geyser-neon-filter")
        .version("1.0")
        .about("Neonlabs filtering service")
        .arg(
            Arg::new("config")
                .short('c')
                .required(false)
                .long("config")
                .value_name("Config path")
                .help("Sets the path to the config file"),
        )
        .get_matches();

    let logger: &'static Logger = fast_log::init(Config::new().console().file_split(
        "/var/log/neon/filter.log",
        LogSize::KB(512),
        RollingType::All,
        LogPacker {},
    ))
    .expect("Failed to initialize fast_log");

    logger.set_level(log::LevelFilter::Debug);

    if let Some(config_path) = app.get_one::<String>("config") {
        println!("Trying to read the config file: {config_path}");

        let contents = fs::read_to_string(config_path)
            .await
            .unwrap_or_else(|_| panic!("Failed to read config: {config_path}"));

        let result: serde_json::Result<FilterConfig> = serde_json::from_str(&contents);
        match result {
            Ok(config) => {
                run(config, logger).await;
            }
            Err(e) => {
                eprintln!("Failed to parse filter config, error {e}");
                error!("Failed to parse filter config, error {e}");
            }
        }
    } else {
        run(env_build_config(), logger).await;
    }
}
