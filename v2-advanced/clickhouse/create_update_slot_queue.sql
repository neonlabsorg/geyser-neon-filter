CREATE TABLE IF NOT EXISTS events.update_slot_queue (
    slot UInt64,
    parent Nullable(UInt64) default 0,
    slot_status Enum('Processed' = 1, 'Rooted' = 2, 'Confirmed' = 3)
)   ENGINE = Kafka SETTINGS
    kafka_broker_list = 'kafka:29092',
    kafka_topic_list = 'update_slot',
    kafka_group_name = 'clickhouse',
    kafka_num_consumers = 1,
    kafka_format = 'JSONEachRow';
