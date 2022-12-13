use rdkafka::{consumer::ConsumerContext, ClientContext, Statistics};

pub struct ContextWithStats;

impl ClientContext for ContextWithStats {
    fn stats(&self, _stats: Statistics) {}
}

impl ConsumerContext for ContextWithStats {}
