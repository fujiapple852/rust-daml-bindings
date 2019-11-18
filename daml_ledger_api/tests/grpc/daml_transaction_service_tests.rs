use std::iter::FromIterator;

use futures::future::Future;
use futures::stream::Stream;

use crate::common::ping_pong::*;
use daml_ledger_api::data::event::{DamlEvent, DamlTreeEvent};
use daml_ledger_api::data::filter::DamlTransactionFilter;
use daml_ledger_api::data::offset::DamlLedgerOffsetBoundary;
use daml_ledger_api::data::offset::{DamlLedgerOffset, DamlLedgerOffsetType};
use daml_ledger_api::data::value::DamlValue;

use daml_ledger_api::data::value::DamlRecordField;
use daml_ledger_api::data::DamlTransactionTree;
use daml_ledger_api::service::DamlVerbosity;
use std::collections::HashSet;

/// Submit a create followed by an exercise and observe the events using `get_transaction_trees` which returns Created
/// and Exercised events only.
#[test]
fn test_get_transaction_trees() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let package_id = get_ping_pong_package_id(&ledger_client)?;

    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let parties = [ALICE_PARTY.to_owned(), BOB_PARTY.to_owned()];
    test_create_ping_contract(
        &ledger_client,
        &package_id,
        &application_id,
        &workflow_id,
        &create_test_uuid(COMMAND_ID_PREFIX),
        0,
    )?;
    test_exercise_pong_choice(
        &ledger_client,
        &package_id,
        &application_id,
        &workflow_id,
        &create_test_uuid(COMMAND_ID_PREFIX),
    )?;
    let transactions_future = ledger_client
        .transaction_service()
        .get_transaction_trees(
            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin),
            DamlLedgerOffsetType::Unbounded,
            DamlTransactionFilter::for_parties(&parties[..]),
            DamlVerbosity::Verbose,
        )?
        .take(2);
    let transactions: Vec<Vec<DamlTransactionTree>> = transactions_future.collect().wait()?;
    let flattened_txs: Vec<&DamlTransactionTree> = transactions.iter().flatten().collect();
    let exercise_tx: &DamlTransactionTree = &flattened_txs[1];
    let ping_exercised_event = match &exercise_tx.events_by_id()["#1:0"] {
        DamlTreeEvent::Exercised(e) => e,
        _ => panic!(),
    };
    let pong_created_event = match &exercise_tx.events_by_id()["#1:1"] {
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
    assert_eq!("#0:0", ping_exercised_event.contract_id());
    assert_eq!(HashSet::<&String>::from_iter(&parties), HashSet::from_iter(ping_exercised_event.witness_parties()));
    assert_eq!(&["#1:1".to_owned()], ping_exercised_event.child_event_ids());
    assert_eq!("RespondPong", ping_exercised_event.choice());
    assert_eq!("#1:0", ping_exercised_event.event_id());
    assert_eq!("Pong", pong_created_event.template_id().entity_name());
    assert_eq!("#1:1", pong_created_event.contract_id());
    assert_eq!(1, count_field_val);
    Ok(())
}

#[test]
fn test_get_transaction_by_event_id() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let package_id = get_ping_pong_package_id(&ledger_client)?;

    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    test_create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0)?;
    let transaction =
        ledger_client.transaction_service().get_transaction_by_event_id_sync("#0:0".to_string(), parties)?;
    let event = match &transaction.events_by_id()["#0:0"] {
        DamlTreeEvent::Created(e) => e,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert_eq!(&["#0:0".to_owned()], transaction.root_event_ids());
    assert_eq!("#0:0", event.event_id());
    assert_eq!(package_id, event.template_id().package_id());
    assert_eq!(PINGPONG_MODULE_NAME, event.template_id().module_name());
    assert_eq!(PING_ENTITY_NAME, event.template_id().entity_name());
    assert_eq!(HashSet::<&String>::from_iter(parties), HashSet::from_iter(event.witness_parties()));
    assert_eq!("#0:0", event.contract_id());
    assert_eq!(
        &create_test_pp_id(&package_id, PING_ENTITY_NAME),
        event.create_arguments().record_id().as_ref().ok_or(ERR_STR)?
    );
    Ok(())
}

#[test]
fn test_get_transaction_by_id() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let package_id = get_ping_pong_package_id(&ledger_client)?;

    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    test_create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0)?;
    let transaction = ledger_client.transaction_service().get_transaction_by_id_sync("0".to_string(), parties)?;
    let event = match &transaction.events_by_id()["#0:0"] {
        DamlTreeEvent::Created(e) => e,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert_eq!(&["#0:0".to_owned()], transaction.root_event_ids());
    assert_eq!("#0:0", event.event_id());
    assert_eq!(package_id, event.template_id().package_id());
    assert_eq!(PINGPONG_MODULE_NAME, event.template_id().module_name());
    assert_eq!(PING_ENTITY_NAME, event.template_id().entity_name());
    assert_eq!(HashSet::<&String>::from_iter(parties), HashSet::from_iter(event.witness_parties()));
    assert_eq!("#0:0", event.contract_id());
    assert_eq!(
        &create_test_pp_id(&package_id, PING_ENTITY_NAME),
        event.create_arguments().record_id().as_ref().ok_or(ERR_STR)?
    );
    Ok(())
}

#[test]
fn test_get_flat_transaction_by_event_id() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let package_id = get_ping_pong_package_id(&ledger_client)?;

    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    test_create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0)?;
    let transaction =
        ledger_client.transaction_service().get_flat_transaction_by_event_id_sync("#0:0".to_string(), parties)?;
    let event = match &transaction.events() {
        [DamlEvent::Created(e)] => e,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert_eq!("#0:0", event.event_id());
    assert_eq!(package_id, event.template_id().package_id());
    assert_eq!(PINGPONG_MODULE_NAME, event.template_id().module_name());
    assert_eq!(PING_ENTITY_NAME, event.template_id().entity_name());
    assert_eq!(HashSet::<&String>::from_iter(parties), HashSet::from_iter(event.witness_parties()));
    assert_eq!("#0:0", event.contract_id());
    assert_eq!(
        &create_test_pp_id(&package_id, PING_ENTITY_NAME),
        event.create_arguments().record_id().as_ref().ok_or(ERR_STR)?
    );
    Ok(())
}

#[test]
fn test_get_flat_transaction_by_id() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let package_id = get_ping_pong_package_id(&ledger_client)?;

    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    test_create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0)?;
    let transaction = ledger_client.transaction_service().get_flat_transaction_by_id_sync("0".to_string(), parties)?;
    let event = match &transaction.events() {
        [DamlEvent::Created(e)] => e,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert_eq!("#0:0", event.event_id());
    assert_eq!(package_id, event.template_id().package_id());
    assert_eq!(PINGPONG_MODULE_NAME, event.template_id().module_name());
    assert_eq!(PING_ENTITY_NAME, event.template_id().entity_name());
    assert_eq!(HashSet::<&String>::from_iter(parties), HashSet::from_iter(event.witness_parties()));
    assert_eq!("#0:0", event.contract_id());
    assert_eq!(
        &create_test_pp_id(&package_id, PING_ENTITY_NAME),
        event.create_arguments().record_id().as_ref().ok_or(ERR_STR)?
    );
    Ok(())
}

#[test]
fn test_get_ledger_end() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let ledger_end_offset = ledger_client.transaction_service().get_ledger_end_sync()?;
    assert_eq!(DamlLedgerOffset::Absolute(0), ledger_end_offset);
    Ok(())
}
