#![warn(clippy::all, clippy::pedantic)]

use daml::prelude::DamlExerciseCommand;
use daml_ledger_api::data::command::DamlCreateCommand;
use daml_ledger_api::data::DamlResult;
use daml_ledger_api::DamlLedgerClient;
use daml_lf::DamlLfArchivePayload;
use std::error::Error;
use std::sync::Mutex;

pub type TestResult = ::std::result::Result<(), Box<dyn Error>>;

pub const SANDBOX_HOST: &str = "localhost";
pub const SANDBOX_PORT: u16 = 8081;

lazy_static! {
    pub static ref SANDBOX_LOCK: Mutex<()> = Mutex::new(());
}

pub fn new_static_sandbox() -> DamlResult<DamlLedgerClient> {
    DamlLedgerClient::connect(SANDBOX_HOST, SANDBOX_PORT)?.reset_and_wait()
}

pub fn update_exercise_command_package_id_for_testing(
    ledger_client: &DamlLedgerClient,
    mut exercise_command: DamlExerciseCommand,
) -> std::result::Result<DamlExerciseCommand, Box<dyn Error>> {
    exercise_command.template_id.package_id =
        get_package_id_from_ledger(ledger_client, exercise_command.template_id().module_name())?;
    Ok(exercise_command)
}

pub fn update_create_command_package_id_for_testing(
    ledger_client: &DamlLedgerClient,
    mut create_command: DamlCreateCommand,
) -> std::result::Result<DamlCreateCommand, Box<dyn Error>> {
    create_command.template_id.package_id =
        get_package_id_from_ledger(ledger_client, create_command.template_id().module_name())?;
    Ok(create_command)
}

fn get_package_id_from_ledger(
    ledger_client: &DamlLedgerClient,
    module_name: &str,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    fn get_package_payload<'a>(
        ledger_client: &DamlLedgerClient,
        package_id: &'a str,
    ) -> std::result::Result<(&'a str, DamlLfArchivePayload), Box<dyn std::error::Error>> {
        let package = ledger_client.package_service().get_package_sync(package_id)?;
        let archive = DamlLfArchivePayload::from_bytes(package.payload())?;
        Ok((package_id, archive))
    }
    let all_packages = ledger_client.package_service().list_packages_sync()?;
    let all_archives: Vec<(&str, DamlLfArchivePayload)> = all_packages
        .iter()
        .map(|package_id| (get_package_payload(ledger_client, package_id)))
        .collect::<std::result::Result<Vec<(&str, DamlLfArchivePayload)>, _>>()?;
    all_archives
        .iter()
        .find(|(_, archive)| archive.contains_module(module_name))
        .map_or(Err("package could not be found".into()), |(package_id, _)| Ok((*package_id).to_string()))
}
