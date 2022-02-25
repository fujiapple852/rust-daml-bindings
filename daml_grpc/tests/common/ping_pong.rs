use daml::util::DamlSandboxTokenBuilder;
use daml_grpc::data::command::{DamlCommand, DamlCreateAndExerciseCommand, DamlCreateCommand, DamlExerciseCommand};
use daml_grpc::data::value::{DamlRecord, DamlRecordBuilder, DamlValue};
use daml_grpc::data::{DamlIdentifier, DamlMinLedgerTime};
use daml_grpc::DamlGrpcClient;
use daml_grpc::{DamlCommandFactory, DamlGrpcClientBuilder};
use std::error::Error;
use std::sync::Once;
use std::time::Duration;
use tokio::sync::{Mutex, MutexGuard};
use tracing_subscriber::fmt::format::FmtSpan;
use uuid::Uuid;
pub type TestResult = ::std::result::Result<(), Box<dyn Error>>;

pub const PINGPONG_MODULE_NAME: &str = "Fuji.PingPong";
pub const PING_ENTITY_NAME: &str = "Ping";
pub const ALICE_PARTY: &str = "Alice";
pub const BOB_PARTY: &str = "Bob";
pub const COMMAND_ID_PREFIX: &str = "cmd";
pub const SUBMISSION_ID_PREFIX: &str = "cmd";
pub const WORKFLOW_ID_PREFIX: &str = "wf";
pub const APPLICATION_ID_PREFIX: &str = "app";
pub const ERR_STR: &str = "error";
pub const STATIC_SANDBOX_URI: &str = "https://127.0.0.1:8081";
pub const WALLCLOCK_SANDBOX_URI: &str = "https://127.0.0.1:8080";
pub const TOKEN_VALIDITY_SECS: i64 = 60;
pub const CONNECT_TIMEOUT_MS: u64 = 1000;
pub const RESET_TIMEOUT_MS: u64 = 60000;
pub const TIMEOUT_MS: u64 = 60000;
pub const TOKEN_KEY_PATH: &str = "../resources/testing_types_sandbox/.auth_certs/es256.key";
// pub const SERVER_CA_CERT_PATH: &str = "../resources/testing_types_sandbox/.tls_certs/ca.cert";

lazy_static! {
    pub static ref STATIC_SANDBOX_LOCK: Mutex<()> = Mutex::new(());
    pub static ref WALLCLOCK_SANDBOX_LOCK: Mutex<()> = Mutex::new(());
}

static INIT: Once = Once::new();

pub async fn initialize_static() -> MutexGuard<'static, ()> {
    init_tracing();
    STATIC_SANDBOX_LOCK.lock().await
}

pub async fn initialize_wallclock() -> MutexGuard<'static, ()> {
    init_tracing();
    WALLCLOCK_SANDBOX_LOCK.lock().await
}

fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_span_events(FmtSpan::NONE)
            .with_env_filter(
                "daml_grpc::service=debug,daml_grpc::service::daml_package_service=off,daml_grpc::service::\
                 daml_ledger_identity_service=off,daml_grpc::service::daml_reset_service=off",
            )
            .init();
    });
}

pub async fn new_wallclock_sandbox() -> anyhow::Result<DamlGrpcClient> {
    new_sandbox(WALLCLOCK_SANDBOX_URI).await
}

pub async fn new_static_sandbox() -> anyhow::Result<DamlGrpcClient> {
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
    DamlCommandFactory::new(
        workflow_id,
        application_id,
        vec![sending_party.into()],
        vec![],
        None,
        DamlMinLedgerTime::Relative(Duration::from_millis(0)),
    )
}

pub fn create_test_pp_id(pingpong_package_id: &str, entity_name: &str) -> DamlIdentifier {
    DamlIdentifier::new(pingpong_package_id, PINGPONG_MODULE_NAME, entity_name)
}

pub fn create_test_uuid(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

pub async fn create_ping_contract(
    ledger_client: &DamlGrpcClient,
    package_id: &str,
    application_id: &str,
    workflow_id: &str,
    create_command_id: &str,
    count: i64,
) -> TestResult {
    let ping_record = create_test_ping_record(ALICE_PARTY, BOB_PARTY, count);
    let commands_factory = create_test_command_factory(workflow_id, application_id, ALICE_PARTY);
    let template_id = create_test_pp_id(package_id, PING_ENTITY_NAME);
    let create_command = DamlCommand::Create(DamlCreateCommand::new(template_id, ping_record));
    let create_commands = commands_factory.make_command_with_id(create_command, create_command_id);
    ledger_client.command_service().submit_and_wait(create_commands).await?;
    Ok(())
}

pub async fn exercise_pong_choice(
    ledger_client: &DamlGrpcClient,
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
    ledger_client: &DamlGrpcClient,
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

pub fn make_ec256_token() -> anyhow::Result<String> {
    Ok(DamlSandboxTokenBuilder::new_with_duration_secs(TOKEN_VALIDITY_SECS)
        .admin(true)
        .act_as(vec![String::from(ALICE_PARTY), String::from(BOB_PARTY)])
        .read_as(vec![String::from(ALICE_PARTY), String::from(BOB_PARTY)])
        .new_ec256_token(std::fs::read_to_string(TOKEN_KEY_PATH)?)?)
}

async fn new_sandbox(uri: &str) -> anyhow::Result<DamlGrpcClient> {
    let client = DamlGrpcClientBuilder::uri(uri)
        .reset_timeout(Duration::from_millis(RESET_TIMEOUT_MS))
        .connect_timeout(Some(Duration::from_millis(CONNECT_TIMEOUT_MS)))
        .timeout(Duration::from_millis(TIMEOUT_MS))
        .with_auth(make_ec256_token()?)
        // .with_tls(std::fs::read_to_string(SERVER_CA_CERT_PATH)?) // TODO re-enable when CI issue resolved
        .connect()
        .await?;
    Ok(client.reset_and_wait().await?)
}
