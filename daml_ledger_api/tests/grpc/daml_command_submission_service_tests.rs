use crate::common::ping_pong::*;

use daml_ledger_api::data::command::DamlCommand;
use daml_ledger_api::data::command::DamlCreateCommand;
use daml_ledger_api::data::completion::{DamlCompletion, DamlCompletionResponse};
use daml_ledger_api::data::offset::{DamlLedgerOffset, DamlLedgerOffsetBoundary};
use futures::future;
use futures::future::Future;
use futures::stream::Stream;

/// Submit a command using the submission service and confirm successful execution from the completion service.
///
/// The completion service returns a periodic "checkpoint" without any completions and so we first skip these (via
/// `skip_while`) and then take and return the next item (via take(1)) which is then checked for the completion we
/// expect.
#[test]
fn test_command_submission_and_completion() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let package_id = get_ping_pong_package_id(&ledger_client)?;

    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let ping_record = create_test_ping_record(ALICE_PARTY, BOB_PARTY, 0);
    let commands_factory = create_test_command_factory(&workflow_id, &application_id, ALICE_PARTY);
    let ping_template_id = create_test_pp_id(&package_id, PING_ENTITY_NAME);
    let create_command = DamlCommand::Create(DamlCreateCommand::new(ping_template_id, ping_record));
    let commands = commands_factory.make_command_with_id(create_command, command_id);
    let command_id = ledger_client.command_submission_service().submit_request_sync(commands)?;
    let command_completion_future = ledger_client
        .command_completion_service()
        .get_completion_stream(
            application_id,
            vec![ALICE_PARTY.to_owned(), BOB_PARTY.to_owned()],
            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin),
        )?
        .skip_while(|f: &DamlCompletionResponse| future::ok(f.completions().is_empty()))
        .take(1)
        .map(|f: DamlCompletionResponse| f.take_completions().swap_remove(0)); // TODO review this
    let completions: Vec<DamlCompletion> = command_completion_future.collect().wait()?;
    let completion = completions.first().ok_or(ERR_STR)?;
    assert_eq!(&command_id, completion.command_id());
    assert_eq!("0", completion.transaction_id());
    assert_eq!(0, completion.status().code());
    Ok(())
}
