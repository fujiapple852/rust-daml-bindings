use std::str::FromStr;
use std::sync::Arc;

use clap::{crate_description, crate_name, crate_version, Arg, Command};
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;

use daml_bridge::{Bridge, BridgeConfigData};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg_required_else_help(true)
        .arg(
            Arg::new("ledger-uri")
                .long("ledger-uri")
                .short('s')
                .takes_value(true)
                .required(true)
                .value_name("uri")
                .help("The ledger server GRPC uri (i.e. https://127.0.0.1:7575)"),
        )
        .arg(
            Arg::new("ledger-connect-timeout")
                .long("ledger-connect-timeout")
                .takes_value(true)
                .required(false)
                .default_value("5s")
                .value_name("duration")
                .help("The ledger server connection timeout"),
        )
        .arg(
            Arg::new("ledger-timeout")
                .long("ledger-timeout")
                .takes_value(true)
                .required(false)
                .default_value("5s")
                .value_name("duration")
                .help("The ledger server timeout"),
        )
        .arg(
            Arg::new("http-host")
                .long("http-host")
                .takes_value(true)
                .required(false)
                .default_value("127.0.0.1")
                .value_name("host")
                .help("The host the http server should listen on"),
        )
        .arg(
            Arg::new("http-port")
                .long("http-port")
                .takes_value(true)
                .required(true)
                .value_name("port")
                .help("The port the http server should listen on"),
        )
        .arg(
            Arg::new("package-reload-interval")
                .long("package-reload-interval")
                .takes_value(true)
                .default_value("5s")
                .required(false)
                .value_name("interval")
                .help("How frequently the bridge should refresh the Daml packages from the ledger server"),
        )
        .arg(
            Arg::new("bridge-token")
                .long("bridge-token")
                .takes_value(true)
                .required(true)
                .value_name("token")
                .help("The JWT token the bridge will use for package refresh from the ledger server"),
        )
        .arg(
            Arg::new("encode-decimal-as-string")
                .long("encode-decimal-as-string")
                .required(false)
                .help("Sets whether decimal values are encoded as JSON strings"),
        )
        .arg(
            Arg::new("encode-int64-as-string")
                .long("encode-int64-as-string")
                .required(false)
                .help("Sets whether int64 values are encoded as JSON strings"),
        )
        .arg(
            Arg::new("log-filter")
                .long("log-filter")
                .required(false)
                .default_value("daml_bridge=info")
                .help("Sets the log filters"),
        )
        .get_matches();

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NONE)
        .with_env_filter(matches.value_of("log-filter").unwrap().to_string())
        .json()
        .init();

    let config = Arc::new(BridgeConfigData::new(
        matches.value_of("ledger-uri").unwrap().to_string(),
        humantime::parse_duration(matches.value_of("ledger-connect-timeout").unwrap())?,
        humantime::parse_duration(matches.value_of("ledger-timeout").unwrap())?,
        matches.value_of("bridge-token").unwrap().to_string(),
        matches.value_of("http-host").unwrap().to_string(),
        u16::from_str(matches.value_of("http-port").unwrap())?,
        humantime::parse_duration(matches.value_of("package-reload-interval").unwrap())?,
        matches.is_present("encode-int64-as-string"),
        matches.is_present("encode-decimal-as-string"),
    ));

    let bridge = Bridge::new(config);
    info!("Starting");
    bridge.run().await
}
