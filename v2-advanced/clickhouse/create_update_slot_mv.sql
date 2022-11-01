CREATE MATERIALIZED VIEW IF NOT EXISTS events.update_slot_queue_mv to events.update_slot_local
AS
    SELECT slot, parent, slot_status, now64() as timestamp
    FROM events.update_slot_queue;
