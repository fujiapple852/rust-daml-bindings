#![warn(clippy::all, clippy::pedantic)]

use daml_ledger_api::chrono::{DateTime, Utc};
use daml_ledger_api::data::command::DamlCommand;
use daml_ledger_api::data::{DamlResult, DamlTransaction};
use daml_ledger_api::{DamlCommandFactory, DamlLedgerClient};
use std::error::Error;
use std::ops::Add;
use std::sync::Mutex;
use time::Duration;

pub type TestResult = ::std::result::Result<(), Box<dyn Error>>;

pub const SANDBOX_HOST: &str = "localhost";
pub const SANDBOX_PORT: u16 = 8081;

lazy_static! {
    pub static ref SANDBOX_LOCK: Mutex<()> = Mutex::new(());
}

pub fn new_static_sandbox() -> DamlResult<DamlLedgerClient> {
    let client = DamlLedgerClient::connect(SANDBOX_HOST, SANDBOX_PORT)?;
    client.reset_and_wait()
}

pub fn test_submit_command(
    ledger_client: &DamlLedgerClient,
    party: &str,
    command: DamlCommand,
) -> DamlResult<DamlTransaction> {
    let ledger_effective_time: DateTime<Utc> =
        "1970-01-01T00:00:00Z".parse::<DateTime<Utc>>().expect("invalid datetime");
    let maximum_record_time = ledger_effective_time.add(Duration::seconds(30));
    let cf = DamlCommandFactory::new("wf-0", "MyApp", party, ledger_effective_time, maximum_record_time);
    let commands = cf.make_command(command);
    ledger_client.command_service().submit_and_wait_for_transaction_sync(commands)
}
