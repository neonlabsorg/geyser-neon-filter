use std::sync::Arc;

use crate::config::FilterConfig;
use anyhow::Result;
use flume::Receiver;
use kafka_common::kafka_structs::UpdateAccount;
use log::error;
use tokio_postgres::{Client, NoTls};

async fn connect_to_db(config: FilterConfig) -> Result<Client> {
    let (client, connection) =
        tokio_postgres::connect(&config.postgres_connection_str, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Connection error: {}", e);
        }
    });

    Ok(client)
}

async fn filter_and_send(
    _config: FilterConfig,
    _client: Arc<Client>,
    update_account: UpdateAccount,
) -> Result<()> {
    match update_account.account {
        kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_1(_) => {}
        kafka_common::kafka_structs::KafkaReplicaAccountInfoVersions::V0_0_2(_) => {}
    }
    Ok(())
}

pub async fn filter(config: FilterConfig, filter_rx: Receiver<UpdateAccount>) {
    let client = match connect_to_db(config.clone()).await {
        Ok(client) => Arc::new(client),
        Err(e) => panic!("Failed to connect to the database, error {e}"),
    };

    loop {
        if let Ok(update_account) = filter_rx.recv_async().await {
            tokio::spawn(filter_and_send(
                config.clone(),
                client.clone(),
                update_account,
            ));
        }
    }
}
