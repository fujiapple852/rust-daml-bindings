#![warn(clippy::all, clippy::pedantic)]

use daml::prelude::DamlExerciseCommand;
use daml_ledger_api::data::command::DamlCreateCommand;
use daml_ledger_api::data::DamlResult;
use daml_ledger_api::DamlLedgerClient;
use daml_ledger_util::package::find_module_package_id;
use std::error::Error;
use std::sync::Mutex;

pub type TestResult = ::std::result::Result<(), Box<dyn Error>>;

pub const SANDBOX_HOST: &str = "localhost";
pub const SANDBOX_PORT: u16 = 8081;

lazy_static! {
    pub static ref SANDBOX_LOCK: Mutex<()> = Mutex::new(());
}

pub async fn new_static_sandbox_async() -> DamlResult<DamlLedgerClient> {
    let client = DamlLedgerClient::connect(SANDBOX_HOST, SANDBOX_PORT).await?;
    client.reset_and_wait().await
}

pub async fn update_exercise_command_package_id_for_testing(
    ledger_client: &DamlLedgerClient,
    mut exercise_command: DamlExerciseCommand,
) -> std::result::Result<DamlExerciseCommand, Box<dyn Error>> {
    exercise_command.template_id.package_id =
        find_module_package_id(ledger_client, exercise_command.template_id().module_name()).await?;
    Ok(exercise_command)
}

pub async fn update_create_command_package_id_for_testing(
    ledger_client: &DamlLedgerClient,
    mut create_command: DamlCreateCommand,
) -> std::result::Result<DamlCreateCommand, Box<dyn Error>> {
    create_command.template_id.package_id =
        find_module_package_id(ledger_client, create_command.template_id().module_name()).await?;
    Ok(create_command)
}
