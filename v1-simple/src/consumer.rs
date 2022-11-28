use std::sync::Arc;

use flume::Sender;
use log::{error, info};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    message::BorrowedMessage,
    ClientConfig, Message,
};
use serde::Deserialize;

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

pub async fn consumer<T>(config: Arc<FilterConfig>, topic: String, filter_tx: Sender<T>)
where
    T: for<'a> Deserialize<'a> + std::marker::Send + 'static,
{
    let type_name = std::any::type_name::<T>();
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", &config.kafka_consumer_group_id)
        .set("bootstrap.servers", &config.bootstrap_servers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", &config.session_timeout_ms)
        .set("enable.auto.commit", "true")
        .set("security.protocol", &config.security_protocol)
        .set("sasl.mechanism", &config.sasl_mechanism)
        .set("sasl.username", &config.sasl_username)
        .set("sasl.password", &config.sasl_password)
        .set_log_level((&config.kafka_log_level).into())
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&[&topic])
        .unwrap_or_else(|_| panic!("Couldn't subscribe to specified topic with {type_name}"));

    info!("The consumer loop is about to start!");

    loop {
        match consumer.recv().await {
            Ok(message) => {
                if let Some(payload) = extract_from_message(&message) {
                    let result: serde_json::Result<T> = serde_json::from_str(payload);
                    let filter_tx = filter_tx.clone();

                    tokio::spawn(async move {
                        match result {
                            Ok(event) => {
                                if let Err(e) = filter_tx.send_async(event).await {
                                    error!("Failed to send the data {type_name}, error {e}");
                                }
                            }
                            Err(e) => error!("Failed to deserialize {type_name} {e}"),
                        }
                    });
                }
            }
            Err(e) => error!("Kafka consumer error: {}", e),
        };
    }
}
