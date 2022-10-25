use std::sync::Arc;

use crate::{config::FilterConfig, db::DbAccountInfo};
use anyhow::Result;
use crossbeam_queue::SegQueue;
use flume::Receiver;
use kafka_common::kafka_structs::UpdateAccount;
use log::error;

async fn process_account_info(
    config: Arc<FilterConfig>,
    db_queue: Arc<SegQueue<DbAccountInfo>>,
    update_account: UpdateAccount,
) -> Result<()> {
    match &update_account.account {
        kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_1(_) => unimplemented!(),
        kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_2(account_info) => {
            let owner = bs58::encode(&account_info.owner).into_string();
            let pubkey = bs58::encode(&account_info.pubkey).into_string();
            if config.filter_include_pubkeys.contains(&pubkey)
                || config.filter_include_owners.contains(&owner)
            {
                db_queue.push(update_account.try_into()?);
            }
        }
    }
    Ok(())
}

pub async fn filter(
    config: Arc<FilterConfig>,
    db_queue: Arc<SegQueue<DbAccountInfo>>,
    filter_rx: Receiver<UpdateAccount>,
) {
    loop {
        if let Ok(update_account) = filter_rx.recv_async().await {
            let config = config.clone();
            let db_queue = db_queue.clone();

            tokio::spawn(async move {
                if let Err(e) = process_account_info(config, db_queue, update_account).await {
                    error!("Failed to process account info, error: {e}");
                }
            });
        }
    }
}
