### Geyser neon filter
This service filters incoming data from Kafka by account owner and puts the result in the Postgres database.

### Configuration File Format
The service is configured using configuration file. An example
configuration file looks like the following:
```
{
    "bootstrap_servers": "167.235.75.213:9092,159.69.197.26:9092,167.235.151.85:9092",
    "postgres_connection_str": "postgresql://username:password@1.1.1.1:3333",
    "update_account_topic": "update_account",
    "session_timeout_ms": "5000",
    "filter_include_owners" : ["put_base58_string","put_base58_string"],
    "filter_exceptions" : ["put_base58_string","put_base58_string"],
    "rdkafka_log_level": "Info",
    "global_log_level": "Info",
}
```
