use chrono::DateTime;
use chrono::Utc;
use daml_ledger_api::data::command::{
    DamlCommand, DamlCreateAndExerciseCommand, DamlCreateCommand, DamlExerciseCommand,
};
use daml_ledger_api::data::value::{DamlRecord, DamlRecordBuilder, DamlValue};
use daml_ledger_api::data::DamlIdentifier;
use daml_ledger_api::data::DamlResult;
use daml_ledger_api::{DamlCommandFactory, DamlLedgerClientBuilder};
use daml_ledger_api::{DamlLedgerClient, DamlSandboxTokenBuilder};
use std::error::Error;
use std::ops::Add;
use std::time::Duration;
use uuid::Uuid;

pub type TestResult = ::std::result::Result<(), Box<dyn Error>>;

pub const PINGPONG_MODULE_NAME: &str = "DA.PingPong";
pub const PING_ENTITY_NAME: &str = "Ping";
pub const ALICE_PARTY: &str = "Alice";
pub const BOB_PARTY: &str = "Bob";
pub const COMMAND_ID_PREFIX: &str = "cmd";
pub const SUBMISSION_ID_PREFIX: &str = "cmd";
pub const WORKFLOW_ID_PREFIX: &str = "wf";
pub const APPLICATION_ID_PREFIX: &str = "app";
pub const ERR_STR: &str = "error";
pub const EPOCH: &str = "1970-01-01T00:00:00Z";
pub const STATIC_SANDBOX_URI: &str = "https://127.0.0.1:8081";
pub const WALLCLOCK_SANDBOX_URI: &str = "https://127.0.0.1:8080";
pub const TOKEN_VALIDITY_SECS: i64 = 60;
pub const CONNECT_TIMEOUT_MS: u64 = 20000;
pub const TOKEN_KEY_PATH: &str = "../resources/testing_types_sandbox/certs/ec256.key";

lazy_static! {
    pub static ref STATIC_SANDBOX_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
    pub static ref WALLCLOCK_SANDBOX_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}

pub async fn new_wallclock_sandbox() -> DamlResult<DamlLedgerClient> {
    new_sandbox(WALLCLOCK_SANDBOX_URI).await
}

pub async fn new_static_sandbox() -> DamlResult<DamlLedgerClient> {
    new_sandbox(STATIC_SANDBOX_URI).await
}

pub fn create_test_ping_record(sender: &str, receiver: &str, count: i64) -> DamlRecord {
    DamlRecordBuilder::new()
        .add_field("sender", DamlValue::new_party(sender))
        .add_field("receiver", DamlValue::new_party(receiver))
        .add_field("count", DamlValue::new_int64(count))
        .build()
}

pub fn create_test_command_factory(workflow_id: &str, application_id: &str, sending_party: &str) -> DamlCommandFactory {
    let ledger_effective_time: DateTime<Utc> = EPOCH.parse::<DateTime<Utc>>().expect("invalid datetime");
    let maximum_record_time = ledger_effective_time.add(chrono::Duration::seconds(30));
    DamlCommandFactory::new(
        workflow_id,
        application_id,
        sending_party,
        ledger_effective_time,
        maximum_record_time,
        None,
    )
}

pub fn create_test_pp_id(pingpong_package_id: &str, entity_name: &str) -> DamlIdentifier {
    DamlIdentifier::new(pingpong_package_id, PINGPONG_MODULE_NAME, entity_name)
}

pub fn create_test_uuid(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

pub async fn create_ping_contract(
    ledger_client: &DamlLedgerClient,
    package_id: &str,
    application_id: &str,
    workflow_id: &str,
    create_command_id: &str,
    count: i64,
) -> TestResult {
    let ping_record = create_test_ping_record(ALICE_PARTY, BOB_PARTY, count);
    let commands_factory = create_test_command_factory(workflow_id, application_id, ALICE_PARTY);
    let template_id = create_test_pp_id(&package_id, PING_ENTITY_NAME);
    let create_command = DamlCommand::Create(DamlCreateCommand::new(template_id, ping_record));
    let create_commands = commands_factory.make_command_with_id(create_command, create_command_id);
    ledger_client.command_service().submit_and_wait(create_commands).await?;
    Ok(())
}

pub async fn exercise_pong_choice(
    ledger_client: &DamlLedgerClient,
    package_id: &str,
    application_id: &str,
    workflow_id: &str,
    exercise_command_id: &str,
    contract_id: &str,
) -> TestResult {
    let template_id = create_test_pp_id(package_id, PING_ENTITY_NAME);
    let bob_commands_factory = create_test_command_factory(workflow_id, application_id, BOB_PARTY);
    let exercise_command = DamlCommand::Exercise(DamlExerciseCommand::new(
        template_id,
        contract_id,
        "RespondPong",
        DamlValue::new_record(DamlRecord::empty()),
    ));
    let exercise_commands = bob_commands_factory.make_command_with_id(exercise_command, exercise_command_id);
    ledger_client.command_service().submit_and_wait(exercise_commands).await?;
    Ok(())
}

pub async fn test_create_ping_and_exercise_reset_ping(
    ledger_client: &DamlLedgerClient,
    package_id: &str,
    application_id: &str,
    workflow_id: &str,
    command_id: &str,
) -> TestResult {
    let ping_record = create_test_ping_record(ALICE_PARTY, BOB_PARTY, 0);
    let commands_factory = create_test_command_factory(workflow_id, application_id, ALICE_PARTY);
    let template_id = create_test_pp_id(package_id, PING_ENTITY_NAME);

    let create_and_exercise_command = DamlCommand::CreateAndExercise(DamlCreateAndExerciseCommand::new(
        template_id,
        ping_record,
        "ResetPingCount",
        DamlValue::new_record(DamlRecord::empty()),
    ));

    let commands = commands_factory.make_command_with_id(create_and_exercise_command, command_id);
    ledger_client.command_service().submit_and_wait(commands).await?;
    Ok(())
}

async fn new_sandbox(uri: &str) -> DamlResult<DamlLedgerClient> {
    let client = DamlLedgerClientBuilder::uri(uri)
        .timeout(Duration::from_millis(CONNECT_TIMEOUT_MS))
        .with_auth(make_ec256_token()?)
        .connect()
        .await?;
    client.reset_and_wait().await
}

fn make_ec256_token() -> DamlResult<String> {
    Ok(DamlSandboxTokenBuilder::new_with_duration_secs(TOKEN_VALIDITY_SECS)
        .admin(true)
        .act_as(vec![String::from(ALICE_PARTY), String::from(BOB_PARTY)])
        .read_as(vec![String::from(ALICE_PARTY), String::from(BOB_PARTY)])
        .new_ec256_token(std::fs::read_to_string(TOKEN_KEY_PATH)?)?)
}
