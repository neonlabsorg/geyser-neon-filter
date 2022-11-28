use anyhow::anyhow;
use anyhow::Result;
use std::sync::Arc;
use tokio_postgres::{Client, Statement};

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

pub async fn create_block_metadata_insert_statement(client: Arc<Client>) -> Result<Statement> {
    let stmt =
        "INSERT INTO block (slot, blockhash, rewards, block_time, block_height, updated_on) \
    VALUES ($1, $2, $3, $4, $5, $6)";

    let stmt = client.prepare(stmt).await;

    match stmt {
        Ok(notify_block_metadata_stmt) => Ok(notify_block_metadata_stmt),
        Err(err) => Err(anyhow!(err)),
    }
}

pub async fn create_slot_insert_statement_with_parent(client: Arc<Client>) -> Result<Statement> {
    let stmt = "INSERT INTO slot (slot, parent, status, updated_on) \
    VALUES ($1, $2, $3, $4) \
    ON CONFLICT (slot) DO UPDATE SET parent=excluded.parent, status=excluded.status, updated_on=excluded.updated_on";

    let stmt = client.prepare(stmt).await;

    match stmt {
        Ok(notify_block_metadata_stmt) => Ok(notify_block_metadata_stmt),
        Err(err) => Err(anyhow!(err)),
    }
}

pub async fn create_slot_insert_statement_without_parent(client: Arc<Client>) -> Result<Statement> {
    let stmt = "INSERT INTO slot (slot, status, updated_on) \
    VALUES ($1, $2, $3) \
    ON CONFLICT (slot) DO UPDATE SET status=excluded.status, updated_on=excluded.updated_on";

    let stmt = client.prepare(stmt).await;

    match stmt {
        Ok(notify_block_metadata_stmt) => Ok(notify_block_metadata_stmt),
        Err(err) => Err(anyhow!(err)),
    }
}
