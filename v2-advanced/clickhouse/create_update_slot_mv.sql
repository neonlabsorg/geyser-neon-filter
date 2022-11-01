CREATE MATERIALIZED VIEW update_slot_queue_mv to update_slot
AS
    SELECT slot, parent, slot_status
    FROM update_slot_queue;
