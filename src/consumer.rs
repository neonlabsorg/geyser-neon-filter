use flume::Sender;
use kafka_common::kafka_structs::UpdateAccount;
use log::error;
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{Consumer, StreamConsumer},
    message::BorrowedMessage,
    ClientConfig, Message,
};

use crate::config::FilterConfig;

pub fn extract_from_message<'a>(message: &'a BorrowedMessage<'a>) -> Option<&'a str> {
    let payload = match message.payload_view::<str>() {
        None => None,
        Some(Ok(s)) => Some(s),
        Some(Err(e)) => {
            error!("Error while deserializing message payload: {:?}", e);
            None
        }
    };
    payload
}

pub async fn consumer(config: FilterConfig, filter_tx: Sender<UpdateAccount>) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &config.bootstrap_servers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", &config.session_timeout_ms)
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Info)
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[&config.update_account_topic])
        .expect("Couldn't subscribe to specified topic");

    loop {
        match consumer.recv().await {
            Ok(message) => {
                if let Some(payload) = extract_from_message(&message) {
                    let result: serde_json::Result<UpdateAccount> = serde_json::from_str(payload);
                    match result {
                        Ok(update_account) => {
                            if let Err(e) = filter_tx.send_async(update_account).await {
                                error!("Failed to send the data, error {e}");
                            }
                        }
                        Err(e) => error!("Failed to deserialize UpdateAccount {e}"),
                    }
                }
            }
            Err(e) => error!("Kafka consumer error: {}", e),
        };
    }
}
