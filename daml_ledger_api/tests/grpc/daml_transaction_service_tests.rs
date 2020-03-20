use crate::common::ping_pong::*;
use daml_ledger_api::data::event::{DamlEvent, DamlTreeEvent};
use daml_ledger_api::data::filter::DamlTransactionFilter;
use daml_ledger_api::data::offset::{DamlLedgerOffset, DamlLedgerOffsetBoundary, DamlLedgerOffsetType};
use daml_ledger_api::data::value::{DamlRecordField, DamlValue};
use daml_ledger_api::data::DamlTransactionTree;
use daml_ledger_api::service::DamlVerbosity;
use daml_ledger_util::package::find_module_package_id;
use futures::StreamExt;
use std::collections::HashSet;
use std::iter::FromIterator;

/// Submit a create followed by an exercise and observe the events using `get_transaction_trees` which returns Created
/// and Exercised events only.
#[tokio::test]
async fn test_get_transaction_trees() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let parties = [ALICE_PARTY.to_owned(), BOB_PARTY.to_owned()];
    create_ping_contract(
        &ledger_client,
        &package_id,
        &application_id,
        &workflow_id,
        &create_test_uuid(COMMAND_ID_PREFIX),
        0,
    )
    .await?;
    let mut transaction_stream = ledger_client
        .transaction_service()
        .get_transaction_trees(
            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin),
            DamlLedgerOffsetType::Unbounded,
            DamlTransactionFilter::for_parties(&parties[..]),
            DamlVerbosity::Verbose,
        )
        .await?;
    let created_transactions: Vec<DamlTransactionTree> =
        transaction_stream.next().await.expect("created transaction")?;
    let create_tx = &created_transactions[0];
    let ping_created_event = match &create_tx.events_by_id()["#1:0"] {
        DamlTreeEvent::Created(e) => e,
        _ => panic!(),
    };
    exercise_pong_choice(
        &ledger_client,
        &package_id,
        &application_id,
        &workflow_id,
        &create_test_uuid(COMMAND_ID_PREFIX),
        &ping_created_event.contract_id,
    )
    .await?;
    let exercised_transactions: Vec<DamlTransactionTree> =
        transaction_stream.next().await.expect("exercised transaction")?;
    let exercise_tx = &exercised_transactions[0];
    let ping_exercised_event = match &exercise_tx.events_by_id()["#2:0"] {
        DamlTreeEvent::Exercised(e) => e,
        _ => panic!(),
    };
    let pong_created_event = match &exercise_tx.events_by_id()["#2:1"] {
        DamlTreeEvent::Created(e) => e,
        _ => panic!(),
    };
    let count_fields: Vec<&DamlRecordField> = pong_created_event
        .create_arguments()
        .fields()
        .iter()
        .filter(|rec| match rec.label() {
            Some(label) if label == "count" => true,
            _ => false,
        })
        .collect();
    let count_field = count_fields.first().ok_or(ERR_STR)?;
    let count_field_val = match *count_field.value() {
        DamlValue::Int64(i) => i,
        _ => panic!(),
    };
    assert_eq!(2, exercise_tx.events_by_id().len());
    assert_eq!(true, ping_exercised_event.consuming());
    assert_eq!(&create_test_pp_id(&package_id, PING_ENTITY_NAME), ping_exercised_event.template_id());
    assert!(!ping_exercised_event.contract_id().is_empty());
    assert_eq!(HashSet::<&String>::from_iter(&parties), HashSet::from_iter(ping_exercised_event.witness_parties()));
    assert_eq!(&["#2:1".to_owned()], ping_exercised_event.child_event_ids());
    assert_eq!("RespondPong", ping_exercised_event.choice());
    assert_eq!("#2:0", ping_exercised_event.event_id());
    assert_eq!("Pong", pong_created_event.template_id().entity_name());
    assert!(!pong_created_event.contract_id().is_empty());
    assert_eq!(1, count_field_val);
    Ok(())
}

#[tokio::test]
async fn test_get_transaction_by_event_id() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0).await?;
    let transaction = ledger_client.transaction_service().get_transaction_by_event_id("#1:0", parties).await?;
    let event = match &transaction.events_by_id()["#1:0"] {
        DamlTreeEvent::Created(e) => e,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert_eq!(&["#1:0".to_owned()], transaction.root_event_ids());
    assert_eq!("#1:0", event.event_id());
    assert_eq!(package_id, event.template_id().package_id());
    assert_eq!(PINGPONG_MODULE_NAME, event.template_id().module_name());
    assert_eq!(PING_ENTITY_NAME, event.template_id().entity_name());
    assert_eq!(HashSet::<&String>::from_iter(parties), HashSet::from_iter(event.witness_parties()));
    assert!(!event.contract_id().is_empty());
    assert_eq!(
        &create_test_pp_id(&package_id, PING_ENTITY_NAME),
        event.create_arguments().record_id().as_ref().ok_or(ERR_STR)?
    );
    Ok(())
}

#[tokio::test]
async fn test_get_transaction_by_id() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0).await?;
    let transaction = ledger_client.transaction_service().get_transaction_by_id("1", parties).await?;
    let event = match &transaction.events_by_id()["#1:0"] {
        DamlTreeEvent::Created(e) => e,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert_eq!(&["#1:0".to_owned()], transaction.root_event_ids());
    assert_eq!("#1:0", event.event_id());
    assert_eq!(package_id, event.template_id().package_id());
    assert_eq!(PINGPONG_MODULE_NAME, event.template_id().module_name());
    assert_eq!(PING_ENTITY_NAME, event.template_id().entity_name());
    assert_eq!(HashSet::<&String>::from_iter(parties), HashSet::from_iter(event.witness_parties()));
    assert!(!event.contract_id().is_empty());
    assert_eq!(
        &create_test_pp_id(&package_id, PING_ENTITY_NAME),
        event.create_arguments().record_id().as_ref().ok_or(ERR_STR)?
    );
    Ok(())
}

#[tokio::test]
async fn test_get_flat_transaction_by_event_id() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0).await?;
    let transaction = ledger_client.transaction_service().get_flat_transaction_by_event_id("#1:0", parties).await?;
    let event = match &transaction.events() {
        [DamlEvent::Created(e)] => e,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert_eq!("#1:0", event.event_id());
    assert_eq!(package_id, event.template_id().package_id());
    assert_eq!(PINGPONG_MODULE_NAME, event.template_id().module_name());
    assert_eq!(PING_ENTITY_NAME, event.template_id().entity_name());
    assert_eq!(HashSet::<&String>::from_iter(parties), HashSet::from_iter(event.witness_parties()));
    assert!(!event.contract_id().is_empty());
    assert_eq!(
        &create_test_pp_id(&package_id, PING_ENTITY_NAME),
        event.create_arguments().record_id().as_ref().ok_or(ERR_STR)?
    );
    Ok(())
}

#[tokio::test]
async fn test_get_flat_transaction_by_id() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0).await?;
    let transaction = ledger_client.transaction_service().get_flat_transaction_by_id("1", parties).await?;
    let event = match &transaction.events() {
        [DamlEvent::Created(e)] => e,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert_eq!("#1:0", event.event_id());
    assert_eq!(package_id, event.template_id().package_id());
    assert_eq!(PINGPONG_MODULE_NAME, event.template_id().module_name());
    assert_eq!(PING_ENTITY_NAME, event.template_id().entity_name());
    assert_eq!(HashSet::<&String>::from_iter(parties), HashSet::from_iter(event.witness_parties()));
    assert!(!event.contract_id().is_empty());
    assert_eq!(
        &create_test_pp_id(&package_id, PING_ENTITY_NAME),
        event.create_arguments().record_id().as_ref().ok_or(ERR_STR)?
    );
    Ok(())
}

#[tokio::test]
async fn test_get_ledger_end() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let ledger_end_offset = ledger_client.transaction_service().get_ledger_end().await?;
    assert_eq!(DamlLedgerOffset::Absolute(1), ledger_end_offset);
    Ok(())
}
