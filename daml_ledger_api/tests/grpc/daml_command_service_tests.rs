use crate::common::ping_pong::*;
use daml_ledger_api::data::command::{DamlCommand, DamlCreateCommand};
use daml_ledger_api::data::event::{DamlEvent, DamlTreeEvent};
use daml_ledger_api::data::filter::DamlTransactionFilter;
use daml_ledger_api::data::offset::{DamlLedgerOffset, DamlLedgerOffsetBoundary, DamlLedgerOffsetType};
use daml_ledger_api::data::{DamlCommands, DamlTransaction};
use daml_ledger_api::service::DamlVerbosity;
use daml_ledger_util::package::find_module_package_id;
use futures::StreamExt;
use futures::TryStreamExt;

#[tokio::test]
async fn test_submit_and_wait_for_create() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let commands = make_commands(&package_id);
    let command_id = commands.command_id().to_owned();
    let submitted_command_id = ledger_client.command_service().submit_and_wait(commands).await?;
    assert_eq!(submitted_command_id, command_id);
    Ok(())
}

#[tokio::test]
async fn test_submit_and_wait_for_transaction_id() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let commands = make_commands(&package_id);
    let transaction_id = ledger_client.command_service().submit_and_wait_for_transaction_id(commands).await?;
    assert!(!transaction_id.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_submit_and_wait_for_transaction() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let commands = make_commands(&package_id);
    let transaction = ledger_client.command_service().submit_and_wait_for_transaction(commands).await?;
    match transaction.events() {
        [DamlEvent::Created(e)] => {
            assert_eq!("Ping", e.template_id().entity_name());
        },
        _ => panic!(),
    }
    Ok(())
}

#[tokio::test]
async fn test_submit_and_wait_for_transaction_tree() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let commands = make_commands(&package_id);
    let transaction = ledger_client.command_service().submit_and_wait_for_transaction_tree(commands).await?;
    match transaction.events_by_id().values().collect::<Vec<_>>().as_slice() {
        [DamlTreeEvent::Created(e)] => {
            assert_eq!("Ping", e.template_id().entity_name());
        },
        _ => panic!(),
    }
    Ok(())
}

/// Test that we are able to retrieve the current ledger offset.
#[tokio::test]
async fn test_completion_end_after_no_commands() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let completion_offset = ledger_client.command_completion_service().get_completion_end().await?;
    assert!(matches!(completion_offset, DamlLedgerOffset::Absolute(_)));
    Ok(())
}

/// Submit a create command (template Ping) as Alice then exercise a choice (Pong) as Bob and observe the archiving
/// of the Ping contract and the creation of the Pong contract.
#[tokio::test]
async fn test_create_contract_and_exercise_choice() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let create_command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let exercise_command_id = create_test_uuid(COMMAND_ID_PREFIX);
    create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &create_command_id, 0).await?;
    let mut transactions_stream = ledger_client
        .transaction_service()
        .get_transactions(
            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin),
            DamlLedgerOffsetType::Unbounded,
            DamlTransactionFilter::for_parties(vec![ALICE_PARTY.to_string(), BOB_PARTY.to_string()]),
            DamlVerbosity::Verbose,
        )
        .await?;
    let created_transactions: Vec<DamlTransaction> = transactions_stream.next().await.expect("created transaction")?;
    let create_tx: &DamlTransaction = &created_transactions[0];
    let ping_created_event = match create_tx.events().first().ok_or(ERR_STR)? {
        DamlEvent::Created(e) => e,
        _ => panic!(),
    };
    exercise_pong_choice(
        &ledger_client,
        &package_id,
        &application_id,
        &workflow_id,
        &exercise_command_id,
        ping_created_event.contract_id(),
    )
    .await?;
    let exercised_transactions: Vec<DamlTransaction> =
        transactions_stream.next().await.expect("exercised transaction")?;
    let exercise_tx: &DamlTransaction = &exercised_transactions[0];
    let ping_archived_event = match &exercise_tx.events()[0] {
        DamlEvent::Archived(e) => e,
        _ => panic!(),
    };
    let pong_created_event = match &exercise_tx.events()[1] {
        DamlEvent::Created(e) => e,
        _ => panic!(),
    };
    assert_eq!(&create_command_id, create_tx.command_id());
    assert_eq!(&exercise_command_id, exercise_tx.command_id());
    assert_eq!("Ping", ping_created_event.template_id().entity_name());
    assert_eq!("Ping", ping_archived_event.template_id().entity_name());
    assert_eq!("Pong", pong_created_event.template_id().entity_name());
    Ok(())
}

#[tokio::test]
async fn test_combined_create_and_exercise() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    test_create_ping_and_exercise_reset_ping(&ledger_client, &package_id, &application_id, &workflow_id, &command_id)
        .await?;
    let transactions_future = ledger_client
        .transaction_service()
        .get_transactions(
            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin),
            DamlLedgerOffsetType::Unbounded,
            DamlTransactionFilter::for_parties(vec![ALICE_PARTY.to_string(), BOB_PARTY.to_string()]),
            DamlVerbosity::Verbose,
        )
        .await?;
    let transactions: Vec<Vec<DamlTransaction>> = transactions_future.take(1).try_collect().await?;
    let flattened_txs: Vec<&DamlTransaction> = transactions.iter().flatten().collect();
    let create_tx: &DamlTransaction = flattened_txs.first().ok_or(ERR_STR)?;
    match create_tx.events() {
        [DamlEvent::Created(e)] => {
            assert_eq!("Ping", e.template_id().entity_name());
        },
        _ => panic!(),
    }
    assert_eq!(&command_id, create_tx.command_id());
    Ok(())
}

fn make_commands(package_id: &str) -> DamlCommands {
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let ping_record = create_test_ping_record(ALICE_PARTY, BOB_PARTY, 0);
    let commands_factory = create_test_command_factory(&workflow_id, &application_id, ALICE_PARTY);
    let ping_template_id = create_test_pp_id(package_id, PING_ENTITY_NAME);
    let create_command = DamlCommand::Create(DamlCreateCommand::new(ping_template_id, ping_record));
    commands_factory.make_command_with_id(create_command, command_id)
}
