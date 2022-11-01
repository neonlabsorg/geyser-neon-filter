CREATE TABLE events.events_main ON CLUSTER '{cluster}' AS events.update_slot_local
ENGINE = Distributed('{cluster}', app, events_local, rand());
