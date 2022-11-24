### Geyser neon filter
This service filters incoming data from Kafka by account owner and puts the result in the Postgres database.

### Configuration File Format
By default, the service is configured using a configuration file named filter_config.json.
You can change the path to the config file by command line option **--config** or **-c**
\
An example configuration file looks like the following:
```
{
    "filter_log_path" : "/var/log/neon/filter.log",
    "bootstrap_servers": "167.235.75.213:9092,159.69.197.26:9092,167.235.151.85:9092",
    "postgres_connection_str": "postgresql://username:password@1.1.1.1:3333/neon-db",
    "sasl_username": "username",
    "sasl_password": "password",
    "sasl_mechanism": "SCRAM-SHA-512",
    "security_protocol": "SASL_SSL",
    "kafka_consumer_group_id": "group_id",
    "update_account_topic": "update_account",
    "session_timeout_ms": "45000",
    "filter_include_owners" : ["put_base58_string","put_base58_string"],
    "filter_include_pubkeys" : ["put_base58_string","put_base58_string"],
    "kafka_log_level": "Info",
    "global_log_level": "Info"
}
```
