#![warn(clippy::all, clippy::pedantic)]

use daml::api::data::command::DamlCreateCommand;
use daml::api::data::DamlResult;
use daml::api::{DamlLedgerClient, DamlLedgerClientBuilder, DamlSandboxTokenBuilder};
use daml::prelude::DamlExerciseCommand;
use daml::util::package::find_module_package_id;
use std::error::Error;
use std::sync::Mutex;

pub type TestResult = ::std::result::Result<(), Box<dyn Error>>;

pub const SANDBOX_URI: &str = "http://127.0.0.1:8081";
pub const ALICE_PARTY: &str = "Alice";
pub const BOB_PARTY: &str = "Bob";
pub const TOKEN_VALIDITY_SECS: i64 = 60;
pub const TOKEN_KEY_PATH: &str = "../resources/testing_types_sandbox/.auth_certs/es256.key";
// pub const SERVER_CA_CERT_PATH: &str = "../resources/testing_types_sandbox/.tls_certs/ca.cert";

lazy_static! {
    pub static ref SANDBOX_LOCK: Mutex<()> = Mutex::new(());
}

pub async fn new_static_sandbox() -> DamlResult<DamlLedgerClient> {
    let client = DamlLedgerClientBuilder::uri(SANDBOX_URI)
        // .with_tls(std::fs::read_to_string(SERVER_CA_CERT_PATH)?) // TODO re-enable when CI issue resolved
        .with_auth(make_ec256_token()?)
        .connect()
        .await?;
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

fn make_ec256_token() -> DamlResult<String> {
    DamlSandboxTokenBuilder::new_with_duration_secs(TOKEN_VALIDITY_SECS)
        .admin(true)
        .act_as(vec![String::from(ALICE_PARTY), String::from(BOB_PARTY)])
        .read_as(vec![String::from(ALICE_PARTY), String::from(BOB_PARTY)])
        .new_ec256_token(std::fs::read_to_string(TOKEN_KEY_PATH)?)
}