use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use chrono::Utc;
use crossbeam_queue::SegQueue;
use kafka_common::kafka_structs::UpdateAccount;
use log::error;
use tokio_postgres::NoTls;
use tokio_postgres::{Client, Statement};

use crate::config::FilterConfig;

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

impl fmt::Display for DbAccountInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<UpdateAccount> for DbAccountInfo {
    type Error = anyhow::Error;

    fn try_from(update_account: UpdateAccount) -> Result<Self> {
        match update_account.account {
            kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_1(_) => {
                unimplemented!()
            }
            kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_2(account_info) => {
                if account_info.lamports > std::i64::MAX as u64 {
                    return Err(anyhow!("account_info.lamports greater than std::i64::MAX!"));
                }
                if account_info.rent_epoch > std::i64::MAX as u64 {
                    return Err(anyhow!(
                        "account_info.rent_epoch greater than std::i64::MAX!"
                    ));
                }
                if account_info.write_version > std::i64::MAX as u64 {
                    return Err(anyhow!(
                        "account_info.write_version greater than std::i64::MAX!"
                    ));
                }

                Ok(DbAccountInfo {
                    pubkey: account_info.pubkey,
                    lamports: account_info.lamports as i64,
                    owner: account_info.owner,
                    executable: account_info.executable,
                    rent_epoch: account_info.rent_epoch as i64,
                    data: account_info.data,
                    slot: update_account.slot as i64,
                    write_version: account_info.write_version as i64,
                    txn_signature: account_info.txn_signature.map(|v| v.as_ref().to_vec()),
                })
            }
        }
    }
}

pub async fn initialize_db_client(config: Arc<FilterConfig>) -> Arc<Client> {
    let client;
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    loop {
        match connect_to_db(config.clone()).await {
            Ok(c) => {
                client = c;
                break;
            }
            Err(e) => {
                error!(
                    "Failed to connect to the database: {}, error: {e}",
                    config.postgres_connection_str
                );
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

pub async fn create_account_insert_statement(client: Arc<Client>) -> Result<Statement> {
    let stmt = "INSERT INTO account AS acct (pubkey, slot, owner, lamports, executable, rent_epoch, data, write_version, updated_on, txn_signature) \
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
    ON CONFLICT (pubkey) DO UPDATE SET slot=excluded.slot, owner=excluded.owner, lamports=excluded.lamports, executable=excluded.executable, rent_epoch=excluded.rent_epoch, \
    data=excluded.data, write_version=excluded.write_version, updated_on=excluded.updated_on, txn_signature=excluded.txn_signature  WHERE acct.slot < excluded.slot OR (\
    acct.slot = excluded.slot AND acct.write_version < excluded.write_version)";

    let stmt = client.prepare(stmt).await;

    match stmt {
        Ok(update_account_stmt) => Ok(update_account_stmt),
        Err(err) => Err(anyhow!(err)),
    }
}

pub async fn insert_into_account_audit(
    account: &DbAccountInfo,
    statement: &Statement,
    client: Arc<Client>,
) -> Result<()> {
    let updated_on = Utc::now().naive_utc();
    if let Err(err) = client
        .execute(
            statement,
            &[
                &account.pubkey,
                &account.slot,
                &account.owner,
                &account.lamports,
                &account.executable,
                &account.rent_epoch,
                &account.data,
                &account.write_version,
                &updated_on,
                &account.txn_signature,
            ],
        )
        .await
    {
        return Err(anyhow!("Statement execution failed , error {err}"));
    }

    Ok(())
}

pub async fn db_statement_executor(
    config: Arc<FilterConfig>,
    mut client: Arc<Client>,
    db_queue: Arc<SegQueue<DbAccountInfo>>,
) {
    let mut idle_interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        if client.is_closed() {
            client = initialize_db_client(config.clone()).await;
        }

        if db_queue.is_empty() {
            idle_interval.tick().await;
        }

        if let Some(db_account_info) = db_queue.pop() {
            let client = client.clone();
            let db_queue = db_queue.clone();

            tokio::spawn(async move {
                let statement = match create_account_insert_statement(client.clone()).await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to execute create_account_insert_statement, error {e}");
                        db_queue.push(db_account_info);
                        return;
                    }
                };

                if let Err(error) =
                    insert_into_account_audit(&db_account_info, &statement, client).await
                {
                    error!("Failed to insert the data to account_audit, error: {error}");
                    // Push account_info back to the database queue
                    db_queue.push(db_account_info);
                }
            });
        }
    }
}
