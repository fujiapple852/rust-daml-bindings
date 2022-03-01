use crate::common::ping_pong::{
    create_test_command_factory, create_test_ping_record, create_test_pp_id, create_test_uuid, initialize_static,
    new_static_sandbox, TestResult, ALICE_PARTY, APPLICATION_ID_PREFIX, BOB_PARTY, COMMAND_ID_PREFIX, ERR_STR,
    PINGPONG_MODULE_NAME, PING_ENTITY_NAME, WORKFLOW_ID_PREFIX,
};

use daml_grpc::data::completion::{DamlCompletion, DamlCompletionResponse};
use daml_grpc::data::offset::{DamlLedgerOffset, DamlLedgerOffsetBoundary};

use daml::util::package::find_module_package_id;
use daml_grpc::data::command::{DamlCommand, DamlCreateCommand};
use daml_grpc::data::DamlResult;
use futures::prelude::*;

/// Submit a command using the submission service and confirm successful execution from the completion service.
///
/// The completion service returns a periodic "checkpoint" without any completions and so we first skip these (via
/// `skip_while`) and then take and return the next item (via take(1)) which is then checked for the completion we
/// expect.
#[tokio::test]
async fn test_command_submission_and_completion() -> TestResult {
    let _lock = initialize_static().await;
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let ping_record = create_test_ping_record(ALICE_PARTY, BOB_PARTY, 0);
    let commands_factory = create_test_command_factory(&workflow_id, &application_id, ALICE_PARTY);
    let ping_template_id = create_test_pp_id(&package_id, PING_ENTITY_NAME);
    let create_command = DamlCommand::Create(DamlCreateCommand::new(ping_template_id, ping_record));
    let commands = commands_factory.make_command_with_id(create_command, command_id);
    let command_id = ledger_client.command_submission_service().submit_request(commands).await?;
    let completion_stream = ledger_client
        .command_completion_service()
        .get_completion_stream(
            application_id,
            vec![ALICE_PARTY.to_owned(), BOB_PARTY.to_owned()],
            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin),
        )
        .await?;
    let completions: Vec<DamlCompletion> = completion_stream
        .skip_while(is_skippable_completion)
        .take(1)
        .map(|f: DamlResult<DamlCompletionResponse>| f.map(|resp| resp.take_completions().swap_remove(0)))
        .try_collect()
        .await?;
    let completion = completions.first().ok_or(ERR_STR)?;
    assert_eq!(&command_id, completion.command_id());
    assert!(!completion.transaction_id().is_empty());
    assert_eq!(0, completion.status().code());
    Ok(())
}

// We wish to skip responses which did not error and contain no completions
fn is_skippable_completion(f: &DamlResult<DamlCompletionResponse>) -> impl Future<Output = bool> {
    future::ready(f.as_ref().map(|r| r.completions().is_empty()).unwrap_or(false))
}
