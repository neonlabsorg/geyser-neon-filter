use std::sync::Arc;

use crate::{
    config::FilterConfig,
    db::{DbAccountInfo, DbBlockInfo},
};
use anyhow::Result;
use crossbeam_queue::SegQueue;
use flume::Receiver;
use kafka_common::kafka_structs::{NotifyBlockMetaData, UpdateAccount, UpdateSlotStatus};
use log::{error, trace};

#[inline(always)]
async fn check_account(
    config: Arc<FilterConfig>,
    db_queue: Arc<SegQueue<DbAccountInfo>>,
    update_account: &UpdateAccount,
    owner: &Vec<u8>,
    pubkey: &Vec<u8>,
) -> Result<()> {
    let owner = bs58::encode(owner).into_string();
    let pubkey = bs58::encode(pubkey).into_string();
    if config.filter_include_pubkeys.contains(&pubkey)
        || config.filter_include_owners.contains(&owner)
    {
        db_queue.push(update_account.try_into()?);
        trace!(
            "Add update_account entry to db queue for pubkey {} owner {}",
            pubkey,
            owner
        );
    }
    Ok(())
}

async fn process_account_info(
    config: Arc<FilterConfig>,
    db_queue: Arc<SegQueue<DbAccountInfo>>,
    update_account: UpdateAccount,
) -> Result<()> {
    match &update_account.account {
        // for 1.13.x or earlier
        kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_1(account_info) => {
            check_account(
                config,
                db_queue,
                &update_account,
                &account_info.owner,
                &account_info.pubkey,
            )
            .await?;
        }
        kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_2(account_info) => {
            check_account(
                config,
                db_queue,
                &update_account,
                &account_info.owner,
                &account_info.pubkey,
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn account_filter(
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

pub async fn block_filter(
    db_queue: Arc<SegQueue<DbBlockInfo>>,
    filter_rx: Receiver<NotifyBlockMetaData>,
) {
    loop {
        if let Ok(notify_block_data) = filter_rx.recv_async().await {
            match notify_block_data.block_info {
                kafka_common::kafka_structs::KafkaReplicaBlockInfoVersions::V0_0_1(bi) => {
                    db_queue.push(bi.into());
                }
            }
        }
    }
}

pub async fn slot_filter(
    slot_db_queue: Arc<SegQueue<UpdateSlotStatus>>,
    filter_rx: Receiver<UpdateSlotStatus>,
) {
    loop {
        if let Ok(update_slot) = filter_rx.recv_async().await {
            slot_db_queue.push(update_slot)
        }
    }
}
