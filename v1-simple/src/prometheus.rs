use std::{
    borrow::Cow,
    future::Future,
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
    sync::Arc,
};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use prometheus_client::{encoding::text::encode, registry::Registry};
use tokio::signal::unix::{signal, SignalKind};

use crate::consumer_stats::Stats;

pub async fn start_prometheus(
    stats: Arc<Stats>,
    update_account_topic: Option<String>,
    update_slot_topic: Option<String>,
    notify_block_topic: Option<String>,
    port: u16,
) {
    let mut registry = <Registry>::default();

    registry.register(
        "kafka_bytes_received",
        "How many bytes were received from Kafka cluster",
        stats.kafka_bytes_rx.clone(),
    );

    registry.register(
        "kafka_errors_consumer",
        "How many consumer errors occurred",
        stats.kafka_errors_consumer.clone(),
    );

    registry.register(
        "kafka_errors_deserialize",
        "How many deserialize errors occurred",
        stats.kafka_errors_deserialize.clone(),
    );

    let registry_with_label = registry.sub_registry_with_label((
        Cow::Borrowed("topic"),
        Cow::from(
            update_account_topic
                .as_ref()
                .unwrap_or(&String::new())
                .clone(),
        ),
    ));

    registry_with_label.register(
        "kafka_messages_received",
        "How many UpdateAccount messages have been received",
        stats.kafka_update_account.clone(),
    );

    let registry_with_label = registry.sub_registry_with_label((
        Cow::Borrowed("topic"),
        Cow::from(update_slot_topic.as_ref().unwrap_or(&String::new()).clone()),
    ));

    registry_with_label.register(
        "kafka_messages_received",
        "How many UpdateSlot messages have been received",
        stats.kafka_update_slot.clone(),
    );

    // Not used for now
    let registry_with_label = registry.sub_registry_with_label((
        Cow::Borrowed("topic"),
        Cow::from(String::from("notify_transaction")),
    ));

    registry_with_label.register(
        "kafka_messages_received",
        "How many NotifyTransaction messages have been received",
        stats.kafka_notify_transaction.clone(),
    );

    let registry_with_label = registry.sub_registry_with_label((
        Cow::Borrowed("topic"),
        Cow::from(
            notify_block_topic
                .as_ref()
                .unwrap_or(&String::new())
                .clone(),
        ),
    ));

    registry_with_label.register(
        "kafka_messages_received",
        "How many NotifyBlock messages have been received",
        stats.kafka_notify_block.clone(),
    );

    let metrics_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    start_metrics_server(metrics_addr, registry).await
}

async fn start_metrics_server(metrics_addr: SocketAddr, registry: Registry) {
    let mut shutdown_stream = signal(SignalKind::terminate()).unwrap();

    println!("Starting metrics server on {metrics_addr}");

    let registry = Arc::new(registry);
    Server::bind(&metrics_addr)
        .serve(make_service_fn(move |_conn| {
            let registry = registry.clone();
            async move {
                let handler = make_handler(registry);
                Ok::<_, io::Error>(service_fn(handler))
            }
        }))
        .with_graceful_shutdown(async move {
            shutdown_stream.recv().await;
        })
        .await
        .expect("Failed to bind hyper server with graceful_shutdown");
}

fn make_handler(
    registry: Arc<Registry>,
) -> impl Fn(Request<Body>) -> Pin<Box<dyn Future<Output = io::Result<Response<Body>>> + Send>> {
    // This closure accepts a request and responds with the OpenMetrics encoding of our metrics.
    move |_req: Request<Body>| {
        let reg = registry.clone();
        Box::pin(async move {
            let mut buf = String::new();
            encode(&mut buf, &reg.clone())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                .map(|_| {
                    let body = Body::from(buf);
                    Response::builder()
                        .header(
                            hyper::header::CONTENT_TYPE,
                            "application/openmetrics-text; version=1.0.0; charset=utf-8",
                        )
                        .body(body)
                        .unwrap()
                })
        })
    }
}
