use std::sync::Arc;

use crate::{
    config::FilterConfig,
    db::{build_single_account_insert_statement, DbAccountInfo},
};
use anyhow::Result;
use crossbeam_queue::SegQueue;
use flume::Receiver;
use kafka_common::kafka_structs::UpdateAccount;
use log::error;
use tokio_postgres::{Client, Statement};

async fn process_account_info(
    config: FilterConfig,
    client: Arc<Client>,
    db_queue: Arc<SegQueue<(Statement, DbAccountInfo)>>,
    update_account: UpdateAccount,
) -> Result<()> {
    match &update_account.account {
        kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_1(_) => unimplemented!(),
        kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_2(account_info) => {
            let owner = bs58::encode(&account_info.owner).into_string();
            let pubkey = bs58::encode(&account_info.pubkey).into_string();
            if config.filter_exceptions.contains(&pubkey)
                || config.filter_include_owners.contains(&owner)
            {
                let statement = build_single_account_insert_statement(client.clone()).await?;
                db_queue.push((statement, update_account.try_into()?));
            }
        }
    }
    Ok(())
}

pub async fn filter(
    config: FilterConfig,
    client: Arc<Client>,
    db_queue: Arc<SegQueue<(Statement, DbAccountInfo)>>,
    filter_rx: Receiver<UpdateAccount>,
) {
    loop {
        if let Ok(update_account) = filter_rx.recv_async().await {
            let config = config.clone();
            let db_queue = db_queue.clone();
            let client = client.clone();

            tokio::spawn(async move {
                if let Err(e) = process_account_info(config, client, db_queue, update_account).await
                {
                    error!("Failed to process account info, error: {e}");
                }
            });
        }
    }
}
