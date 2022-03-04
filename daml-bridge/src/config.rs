use std::time::Duration;

/// Daml Bridge configuration.
#[derive(Debug)]
pub struct BridgeConfigData {
    ledger_uri: String,
    ledger_connect_timeout: Duration,
    ledger_timeout: Duration,
    ledger_token: String,
    http_host: String,
    http_port: u16,
    package_reload_interval: Duration,
    encode_int64_as_string: bool,
    encode_decimal_as_string: bool,
}

impl BridgeConfigData {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        ledger_uri: String,
        ledger_connect_timeout: Duration,
        ledger_timeout: Duration,
        ledger_token: String,
        http_host: String,
        http_port: u16,
        package_reload_interval: Duration,
        encode_int64_as_string: bool,
        encode_decimal_as_string: bool,
    ) -> Self {
        Self {
            ledger_uri,
            ledger_connect_timeout,
            ledger_timeout,
            ledger_token,
            http_host,
            http_port,
            package_reload_interval,
            encode_int64_as_string,
            encode_decimal_as_string,
        }
    }

    pub fn ledger_uri(&self) -> &str {
        &self.ledger_uri
    }

    pub const fn ledger_connect_timeout(&self) -> Duration {
        self.ledger_connect_timeout
    }

    pub const fn ledger_timeout(&self) -> Duration {
        self.ledger_timeout
    }

    pub fn ledger_token(&self) -> &str {
        &self.ledger_token
    }

    pub fn http_host(&self) -> &str {
        &self.http_host
    }

    pub const fn http_port(&self) -> u16 {
        self.http_port
    }

    pub const fn package_reload_interval(&self) -> Duration {
        self.package_reload_interval
    }

    pub const fn encode_int64_as_string(&self) -> bool {
        self.encode_int64_as_string
    }

    pub const fn encode_decimal_as_string(&self) -> bool {
        self.encode_decimal_as_string
    }
}
