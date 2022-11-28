use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use crossbeam_queue::SegQueue;
use kafka_common::kafka_structs::KafkaReplicaBlockInfo;
use kafka_common::kafka_structs::UpdateAccount;
use kafka_common::kafka_structs::UpdateSlotStatus;
use log::error;
use log::info;
use log::warn;
use postgres_types::FromSql;
use solana_runtime::bank::RewardType;
use solana_transaction_status::Reward;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;
use tokio_postgres::NoTls;

use crate::config::FilterConfig;
use crate::db_inserts::insert_into_account_audit;
use crate::db_inserts::insert_into_block_metadata;
use crate::db_inserts::insert_slot_status_internal;
use crate::db_statements::create_account_insert_statement;
use crate::db_statements::create_block_metadata_insert_statement;
use crate::db_statements::create_slot_insert_statement_with_parent;
use crate::db_statements::create_slot_insert_statement_without_parent;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DbAccountInfo {
    pub pubkey: Vec<u8>,
    pub lamports: i64,
    pub owner: Vec<u8>,
    pub executable: bool,
    pub rent_epoch: i64,
    pub data: Vec<u8>,
    pub slot: i64,
    pub write_version: i64,
    pub txn_signature: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct DbBlockInfo {
    pub slot: i64,
    pub blockhash: String,
    pub rewards: Vec<DbReward>,
    pub block_time: Option<i64>,
    pub block_height: Option<i64>,
}

#[derive(Clone, Debug, FromSql, ToSql, Eq, PartialEq)]
#[postgres(name = "RewardType")]
pub enum DbRewardType {
    Fee,
    Rent,
    Staking,
    Voting,
}

#[derive(Clone, Debug, FromSql, ToSql)]
#[postgres(name = "Reward")]
pub struct DbReward {
    pub pubkey: String,
    pub lamports: i64,
    pub post_balance: i64,
    pub reward_type: Option<DbRewardType>,
    pub commission: Option<i16>,
}

impl fmt::Display for DbAccountInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn range_check(lamports: u64, rent_epoch: u64, write_version: u64) -> Result<()> {
    if lamports > std::i64::MAX as u64 {
        return Err(anyhow!("account_info.lamports greater than std::i64::MAX!"));
    }
    if rent_epoch > std::i64::MAX as u64 {
        return Err(anyhow!(
            "account_info.rent_epoch greater than std::i64::MAX!"
        ));
    }
    if write_version > std::i64::MAX as u64 {
        return Err(anyhow!(
            "account_info.write_version greater than std::i64::MAX!"
        ));
    }
    Ok(())
}

impl TryFrom<&UpdateAccount> for DbAccountInfo {
    type Error = anyhow::Error;

    fn try_from(update_account: &UpdateAccount) -> Result<Self> {
        match &update_account.account {
            kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_1(account_info) => {
                range_check(
                    account_info.lamports,
                    account_info.rent_epoch,
                    account_info.write_version,
                )?;

                Ok(DbAccountInfo {
                    pubkey: account_info.pubkey.clone(),
                    lamports: account_info.lamports as i64,
                    owner: account_info.owner.clone(),
                    executable: account_info.executable,
                    rent_epoch: account_info.rent_epoch as i64,
                    data: account_info.data.clone(),
                    slot: update_account.slot as i64,
                    write_version: account_info.write_version as i64,
                    txn_signature: None,
                })
            }
            kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_2(account_info) => {
                range_check(
                    account_info.lamports,
                    account_info.rent_epoch,
                    account_info.write_version,
                )?;

                Ok(DbAccountInfo {
                    pubkey: account_info.pubkey.clone(),
                    lamports: account_info.lamports as i64,
                    owner: account_info.owner.clone(),
                    executable: account_info.executable,
                    rent_epoch: account_info.rent_epoch as i64,
                    data: account_info.data.clone(),
                    slot: update_account.slot as i64,
                    write_version: account_info.write_version as i64,
                    txn_signature: account_info.txn_signature.map(|v| v.as_ref().to_vec()),
                })
            }
        }
    }
}

impl From<RewardType> for DbRewardType {
    fn from(reward_type: RewardType) -> Self {
        match reward_type {
            RewardType::Fee => Self::Fee,
            RewardType::Rent => Self::Rent,
            RewardType::Staking => Self::Staking,
            RewardType::Voting => Self::Voting,
        }
    }
}

impl From<&Reward> for DbReward {
    fn from(reward: &Reward) -> Self {
        DbReward {
            pubkey: reward.pubkey.clone(),
            lamports: reward.lamports,
            post_balance: reward.post_balance as i64,
            reward_type: reward.reward_type.map(|v| v.into()),
            commission: reward.commission.map(|v| v as i16),
        }
    }
}

impl From<KafkaReplicaBlockInfo> for DbBlockInfo {
    fn from(block_info: KafkaReplicaBlockInfo) -> Self {
        Self {
            slot: block_info.slot as i64,
            blockhash: block_info.blockhash.to_string(),
            rewards: block_info.rewards.iter().map(DbReward::from).collect(),
            block_time: block_info.block_time,
            block_height: block_info
                .block_height
                .map(|block_height| block_height as i64),
        }
    }
}

pub async fn initialize_db_client(config: Arc<FilterConfig>) -> Arc<Client> {
    let client;
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    loop {
        match connect_to_db(config.clone()).await {
            Ok(c) => {
                info!("A new Postgres client was created and successfully connected to the server");
                client = c;
                break;
            }
            Err(e) => {
                error!("Failed to connect to the database, error: {e}",);
                interval.tick().await;
            }
        };
    }
    client
}

async fn connect_to_db(config: Arc<FilterConfig>) -> Result<Arc<Client>> {
    let (client, connection) =
        tokio_postgres::connect(&config.postgres_connection_str, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Postgres connection error: {}", e);
        }
    });

    Ok(Arc::new(client))
}

pub async fn db_stmt_executor(
    config: Arc<FilterConfig>,
    mut client: Arc<Client>,
    account_queue: Arc<SegQueue<DbAccountInfo>>,
    block_queue: Arc<SegQueue<DbBlockInfo>>,
    slot_queue: Arc<SegQueue<UpdateSlotStatus>>,
) {
    let mut idle_interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        if client.is_closed() {
            warn!("Postgres client was unexpectedly closed");
            client = initialize_db_client(config.clone()).await;
        }

        if account_queue.is_empty() && block_queue.is_empty() && slot_queue.is_empty() {
            idle_interval.tick().await;
        }

        if let Some(db_account_info) = account_queue.pop() {
            let client = client.clone();
            let account_queue = account_queue.clone();

            tokio::spawn(async move {
                let statement = match create_account_insert_statement(client.clone()).await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to execute create_account_insert_statement, error {e}");
                        account_queue.push(db_account_info);
                        return;
                    }
                };

                if let Err(error) =
                    insert_into_account_audit(&db_account_info, &statement, client).await
                {
                    error!("Failed to insert the data to account_audit, error: {error}");
                    // Push account_info back to the database queue
                    account_queue.push(db_account_info);
                }
            });
        }

        if let Some(db_block_info) = block_queue.pop() {
            let client = client.clone();
            let block_queue = block_queue.clone();

            tokio::spawn(async move {
                let statement = match create_block_metadata_insert_statement(client.clone()).await {
                    Ok(s) => s,
                    Err(e) => {
                        error!(
                            "Failed to prepare create_block_metadata_insert_statement, error {e}"
                        );
                        block_queue.push(db_block_info);
                        return;
                    }
                };
                if let Err(error) =
                    insert_into_block_metadata(&db_block_info, &statement, client).await
                {
                    error!("Failed to insert the data to block_metadata, error: {error}");
                    // Push block_info back to the database queue
                    block_queue.push(db_block_info);
                }
            });
        }

        if let Some(db_slot_info) = slot_queue.pop() {
            let client = client.clone();
            let slot_queue = slot_queue.clone();

            tokio::spawn(async move {
                let statement = match db_slot_info.parent {
                    Some(_) => create_slot_insert_statement_with_parent(client.clone()).await,
                    None => create_slot_insert_statement_without_parent(client.clone()).await,
                };

                match statement {
                    Ok(statement) => {
                        if let Err(e) =
                            insert_slot_status_internal(&db_slot_info, &statement, client).await
                        {
                            error!("Failed to execute insert_slot_status_internal, error {e}");
                            slot_queue.push(db_slot_info);
                        }
                    }
                    Err(e) => {
                        error!("Failed to prepare create_slot_insert_statement, error {e}");
                        slot_queue.push(db_slot_info);
                    }
                }
            });
        }
    }
}
