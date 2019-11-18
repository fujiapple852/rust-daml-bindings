use daml_ledger_api::data::offset::DamlLedgerOffset;

use crate::common::ping_pong::*;

/// Submit a create command (blocking server side until complete) and then verify the offset reflects this.
#[test]
fn test_completion_end_after_single_create_command() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let package_id = get_ping_pong_package_id(&ledger_client)?;

    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    test_create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0)?;
    let completion_offset = ledger_client.command_completion_service().get_completion_end_sync()?;
    assert_eq!(DamlLedgerOffset::Absolute(1), completion_offset);
    Ok(())
}
