use clickhouse::{error::Result, Client};

#[tokio::main]
async fn main() -> Result<()> {
    let _client = Client::default().with_url("http://localhost:8123");
    Ok(())
}
