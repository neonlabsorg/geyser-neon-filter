CREATE MATERIALIZED VIEW IF NOT EXISTS update_slot_queue_mv to update_slot
AS
    SELECT slot, parent, slot_status, now64() as timestamp
    FROM update_slot_queue;
