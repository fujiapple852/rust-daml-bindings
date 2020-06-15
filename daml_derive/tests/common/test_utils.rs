use daml::grpc_api::data::command::{DamlCreateCommand, DamlExerciseCommand};
use daml::grpc_api::data::DamlIdentifier;
use daml::grpc_api::{DamlGrpcClient, DamlGrpcClientBuilder};
use daml::util::package::find_module_package_id;
use daml::util::DamlSandboxTokenBuilder;
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

pub async fn new_static_sandbox() -> anyhow::Result<DamlGrpcClient> {
    let client = DamlGrpcClientBuilder::uri(SANDBOX_URI)
        // .with_tls(std::fs::read_to_string(SERVER_CA_CERT_PATH)?) // TODO re-enable when CI issue resolved
        .with_auth(make_ec256_token()?)
        .connect()
        .await?;
    Ok(client.reset_and_wait().await?)
}

pub async fn update_exercise_command_package_id_for_testing(
    ledger_client: &DamlGrpcClient,
    exercise_command: DamlExerciseCommand,
) -> std::result::Result<DamlExerciseCommand, Box<dyn Error>> {
    let new_template_id = find_and_replace_package_id(ledger_client, exercise_command.template_id()).await?;
    let modified_exercise_command = DamlExerciseCommand::new(
        new_template_id,
        exercise_command.contract_id(),
        exercise_command.choice(),
        exercise_command.choice_argument().clone(),
    );
    Ok(modified_exercise_command)
}

pub async fn update_create_command_package_id_for_testing(
    ledger_client: &DamlGrpcClient,
    create_command: DamlCreateCommand,
) -> std::result::Result<DamlCreateCommand, Box<dyn Error>> {
    let new_template_id = find_and_replace_package_id(ledger_client, create_command.template_id()).await?;
    let modified_create_command = DamlCreateCommand::new(new_template_id, create_command.create_arguments().clone());
    Ok(modified_create_command)
}

async fn find_and_replace_package_id(
    ledger_client: &DamlGrpcClient,
    template_id: &DamlIdentifier,
) -> std::result::Result<DamlIdentifier, Box<dyn Error>> {
    let module_package_id = find_module_package_id(ledger_client, template_id.module_name()).await?;
    Ok(DamlIdentifier::new(module_package_id, template_id.module_name(), template_id.entity_name()))
}

fn make_ec256_token() -> anyhow::Result<String> {
    Ok(DamlSandboxTokenBuilder::new_with_duration_secs(TOKEN_VALIDITY_SECS)
        .admin(true)
        .act_as(vec![String::from(ALICE_PARTY), String::from(BOB_PARTY)])
        .read_as(vec![String::from(ALICE_PARTY), String::from(BOB_PARTY)])
        .new_ec256_token(std::fs::read_to_string(TOKEN_KEY_PATH)?)?)
}
