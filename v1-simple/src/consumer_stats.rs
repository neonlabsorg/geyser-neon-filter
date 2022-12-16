use std::sync::{atomic::AtomicU64, Arc};

use log::info;
use prometheus_client::metrics::counter::Counter;
use rdkafka::{consumer::ConsumerContext, ClientContext, Statistics};

#[derive(Default)]
pub struct Stats {
    pub kafka_update_account: Counter<u64, AtomicU64>,
    pub kafka_update_slot: Counter<u64, AtomicU64>,
    pub kafka_notify_transaction: Counter<u64, AtomicU64>,
    pub kafka_notify_block: Counter<u64, AtomicU64>,
    pub kafka_error_consumer: Counter<u64, AtomicU64>,
    pub kafka_error_deserialize: Counter<u64, AtomicU64>,
    pub kafka_bytes_rx: Counter<u64, AtomicU64>,
}

pub trait GetCounters {
    fn get_counters(&self) -> (&AtomicU64, &AtomicU64);
}

#[derive(Default, Clone)]
pub struct ContextWithStats {
    pub stats: Arc<Stats>,
}

impl ClientContext for ContextWithStats {
    fn stats(&self, stats: Statistics) {
        info!("{:?}", stats);
    }
}

impl ConsumerContext for ContextWithStats {}
