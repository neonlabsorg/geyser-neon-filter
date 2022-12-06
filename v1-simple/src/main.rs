mod build_info;
mod config;
mod consumer;
mod db;
mod db_inserts;
mod db_statements;
mod filter;

use std::sync::Arc;

use crate::{
    build_info::get_build_info,
    consumer::consumer,
    db::DbBlockInfo,
    filter::{block_filter, slot_filter},
};
use clap::{Arg, Command};
use config::{env_build_config, FilterConfig};
use crossbeam_queue::SegQueue;
use db::{db_stmt_executor, initialize_db_client, DbAccountInfo};
use fast_log::{
    consts::LogSize,
    plugin::{file_split::RollingType, packer::LogPacker},
    Config, Logger,
};
use filter::account_filter;
use kafka_common::kafka_structs::{NotifyBlockMetaData, UpdateAccount, UpdateSlotStatus};
use log::{error, info};
use tokio::fs;

async fn run(mut config: FilterConfig) {
    let logger: &'static Logger = fast_log::init(Config::new().console().file_split(
        &config.filter_log_path,
        LogSize::KB(512),
        RollingType::All,
        LogPacker {},
    ))
    .expect("Failed to initialize fast_log");

    info!("{}", get_build_info());

    let update_account_topic = config
        .update_account_topic
        .take()
        .expect("update_account_topic is not present in config");

    let notify_block_topic = config
        .notify_block_topic
        .take()
        .expect("notify_block_topic is not present in config");

    let update_slot_topic = config
        .update_slot_topic
        .take()
        .expect("notify_slot_topic is not present in config");

    let config = Arc::new(config);

    let db_account_queue: Arc<SegQueue<DbAccountInfo>> = Arc::new(SegQueue::new());
    let db_block_queue: Arc<SegQueue<DbBlockInfo>> = Arc::new(SegQueue::new());
    let db_slot_queue: Arc<SegQueue<UpdateSlotStatus>> = Arc::new(SegQueue::new());

    logger.set_level((&config.global_log_level).into());

    let client = initialize_db_client(config.clone()).await;

    let (filter_tx_account, filter_rx_account) = flume::unbounded::<UpdateAccount>();
    let (filter_tx_slots, filter_rx_slots) = flume::unbounded::<UpdateSlotStatus>();
    let (filter_tx_block, filter_rx_block) = flume::unbounded::<NotifyBlockMetaData>();

    let account_filter = tokio::spawn(account_filter(
        config.clone(),
        db_account_queue.clone(),
        filter_rx_account,
    ));

    let block_filter = tokio::spawn(block_filter(db_block_queue.clone(), filter_rx_block));

    let slot_filter = tokio::spawn(slot_filter(db_slot_queue.clone(), filter_rx_slots));

    let consumer_update_account = tokio::spawn(consumer(
        config.clone(),
        update_account_topic,
        filter_tx_account,
    ));

    let consumer_update_slot =
        tokio::spawn(consumer(config.clone(), update_slot_topic, filter_tx_slots));

    let consumer_notify_block = tokio::spawn(consumer(
        config.clone(),
        notify_block_topic,
        filter_tx_block,
    ));

    let db_stmt_executor = tokio::spawn(db_stmt_executor(
        config.clone(),
        client,
        db_account_queue,
        db_block_queue,
        db_slot_queue,
    ));

    let _ = tokio::join!(
        consumer_update_account,
        consumer_update_slot,
        consumer_notify_block,
        account_filter,
        block_filter,
        slot_filter,
        db_stmt_executor
    );
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

    println!("{}", get_build_info());

    if let Some(config_path) = app.get_one::<String>("config") {
        println!("Trying to read the config file: {config_path}");

        let contents = fs::read_to_string(config_path)
            .await
            .unwrap_or_else(|e| panic!("Failed to read config: {config_path}, error: {e}"));

        let result: serde_json::Result<FilterConfig> = serde_json::from_str(&contents);
        match result {
            Ok(config) => {
                run(config).await;
            }
            Err(e) => {
                eprintln!("Failed to parse filter config, error {e}");
                error!("Failed to parse filter config, error {e}");
            }
        }
    } else {
        run(env_build_config()).await;
    }
}
