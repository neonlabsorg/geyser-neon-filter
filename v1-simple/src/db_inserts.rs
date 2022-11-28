use anyhow::anyhow;
use anyhow::Result;
use chrono::Utc;
use kafka_common::kafka_structs::UpdateSlotStatus;
use std::sync::Arc;
use tokio_postgres::{Client, Statement};

use crate::db::DbAccountInfo;
use crate::db::DbBlockInfo;

pub async fn insert_into_account_audit(
    account: &DbAccountInfo,
    statement: &Statement,
    client: Arc<Client>,
) -> Result<()> {
    let updated_on = Utc::now().naive_utc();
    if let Err(error) = client
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
        return Err(anyhow!(
            "DbAccountInfo statement execution failed, error {error}"
        ));
    }

    Ok(())
}

pub async fn insert_into_block_metadata(
    block_info: &DbBlockInfo,
    statement: &Statement,
    client: Arc<Client>,
) -> Result<()> {
    let updated_on = Utc::now().naive_utc();

    if let Err(error) = client
        .query(
            statement,
            &[
                &block_info.slot,
                &block_info.blockhash,
                &block_info.rewards,
                &block_info.block_time,
                &block_info.block_height,
                &updated_on,
            ],
        )
        .await
    {
        return Err(anyhow!(
            "DbBlockInfo statement execution failed, error {error}"
        ));
    }

    Ok(())
}

pub async fn insert_slot_status_internal(
    update_slot: &UpdateSlotStatus,
    statement: &Statement,
    client: Arc<Client>,
) -> Result<()> {
    let updated_on = Utc::now().naive_utc();
    let status_str = update_slot.status.to_string();

    let result = match update_slot.parent {
        Some(_) => {
            client
                .execute(
                    statement,
                    &[
                        &(update_slot.slot as i64),
                        &(update_slot.parent.map(|v| v as i64)),
                        &status_str,
                        &updated_on,
                    ],
                )
                .await
        }
        None => {
            client
                .execute(
                    statement,
                    &[&(update_slot.slot as i64), &status_str, &updated_on],
                )
                .await
        }
    };

    if let Err(error) = result {
        return Err(anyhow!(
            "UpdateSlotStatus statement execution failed, error {error}"
        ));
    }

    Ok(())
}
