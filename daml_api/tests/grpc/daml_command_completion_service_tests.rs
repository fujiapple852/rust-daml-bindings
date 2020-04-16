use crate::common::ping_pong::*;
use daml::util::package::find_module_package_id;
use daml_api::data::offset::DamlLedgerOffset;

/// Submit a create command (blocking server side until complete) and then verify the offset reflects this.
#[tokio::test]
async fn test_completion_end_after_single_create_command() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;

    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0).await?;
    let completion_offset = ledger_client.command_completion_service().get_completion_end().await?;
    assert!(matches!(completion_offset, DamlLedgerOffset::Absolute(_)));
    Ok(())
}
