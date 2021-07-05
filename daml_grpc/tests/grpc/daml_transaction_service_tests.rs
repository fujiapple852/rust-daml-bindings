use crate::common::ping_pong::{
    create_ping_contract, create_test_pp_id, create_test_uuid, exercise_pong_choice, initialize_static,
    new_static_sandbox, TestResult, ALICE_PARTY, APPLICATION_ID_PREFIX, BOB_PARTY, COMMAND_ID_PREFIX, ERR_STR,
    PINGPONG_MODULE_NAME, PING_ENTITY_NAME, WORKFLOW_ID_PREFIX,
};
use daml::util::package::find_module_package_id;
use daml_grpc::data::event::{DamlEvent, DamlTreeEvent};
use daml_grpc::data::filter::DamlTransactionFilter;
use daml_grpc::data::offset::{DamlLedgerOffset, DamlLedgerOffsetBoundary, DamlLedgerOffsetType};
use daml_grpc::data::value::{DamlRecordField, DamlValue};
use daml_grpc::data::{DamlError, DamlResult, DamlTransaction, DamlTransactionTree};
use daml_grpc::service::DamlVerbosity;
use daml_grpc::DamlGrpcClient;
use futures::{StreamExt, TryStreamExt};
use std::collections::HashSet;
use std::iter::FromIterator;

/// Submit a create followed by an exercise and observe the events using `get_transaction_trees` which returns Created
/// and Exercised events only.
#[tokio::test]
async fn test_get_transaction_trees() -> TestResult {
    let _lock = initialize_static().await;
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
    let ping_created_event = match create_tx.events_by_id().values().collect::<Vec<_>>().as_slice() {
        [DamlTreeEvent::Created(evt)] => evt,
        _ => panic!(),
    };
    exercise_pong_choice(
        &ledger_client,
        &package_id,
        &application_id,
        &workflow_id,
        &create_test_uuid(COMMAND_ID_PREFIX),
        ping_created_event.contract_id(),
    )
    .await?;
    let exercised_transactions: Vec<DamlTransactionTree> =
        transaction_stream.next().await.expect("exercised transaction")?;
    let exercise_tx = &exercised_transactions[0];
    let (ping_exercised_event, pong_created_event) =
        match exercise_tx.events_by_id().values().collect::<Vec<_>>().as_slice() {
            [DamlTreeEvent::Exercised(exercise), DamlTreeEvent::Created(create)]
            | [DamlTreeEvent::Created(create), DamlTreeEvent::Exercised(exercise)] => (exercise, create),
            _ => panic!(),
        };
    let count_fields: Vec<&DamlRecordField> = pong_created_event
        .create_arguments()
        .fields()
        .iter()
        .filter(|rec| matches!(rec.label(), Some(label) if label == "count"))
        .collect();
    let count_field = count_fields.first().ok_or(ERR_STR)?;
    let count_field_val = match *count_field.value() {
        DamlValue::Int64(i) => i,
        _ => panic!(),
    };
    assert_eq!(2, exercise_tx.events_by_id().len());
    assert!(ping_exercised_event.consuming());
    assert_eq!(&create_test_pp_id(&package_id, PING_ENTITY_NAME), ping_exercised_event.template_id());
    assert!(!ping_exercised_event.contract_id().is_empty());
    assert_eq!(HashSet::<&String>::from_iter(&parties), HashSet::from_iter(ping_exercised_event.witness_parties()));
    assert_eq!(1, ping_exercised_event.child_event_ids().len());
    assert_eq!("RespondPong", ping_exercised_event.choice());
    assert!(!ping_exercised_event.event_id().is_empty());
    assert_eq!("Pong", pong_created_event.template_id().entity_name());
    assert!(!pong_created_event.contract_id().is_empty());
    assert_eq!(1, count_field_val);
    Ok(())
}

#[tokio::test]
async fn test_get_transaction_by_event_id() -> TestResult {
    let _lock = initialize_static().await;
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0).await?;
    let expected_event_id = extract_first_created_event_id(&ledger_client).await?;
    let transaction =
        ledger_client.transaction_service().get_transaction_by_event_id(&expected_event_id, parties).await?;
    let event = match &transaction.events_by_id()[&expected_event_id] {
        DamlTreeEvent::Created(e) => e,
        DamlTreeEvent::Exercised(_) => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert_eq!(&[expected_event_id.clone()], transaction.root_event_ids());
    assert_eq!(expected_event_id, event.event_id());
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
    let _lock = initialize_static().await;
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0).await?;
    let expected_transaction_id = extract_first_transaction_id(&ledger_client).await?;
    let transaction =
        ledger_client.transaction_service().get_transaction_by_id(expected_transaction_id, parties).await?;
    let event = match transaction.events_by_id().values().collect::<Vec<_>>().as_slice() {
        [DamlTreeEvent::Created(evt)] => evt,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert!(!transaction.root_event_ids().is_empty());
    assert!(!event.event_id().is_empty());
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
    let _lock = initialize_static().await;
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0).await?;
    let expected_event_id = extract_first_created_event_id(&ledger_client).await?;

    let transaction =
        ledger_client.transaction_service().get_flat_transaction_by_event_id(&expected_event_id, parties).await?;
    let event = match &transaction.events() {
        [DamlEvent::Created(e)] => e,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert_eq!(&expected_event_id, event.event_id());
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
    let _lock = initialize_static().await;
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let command_id = create_test_uuid(COMMAND_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let parties = &[ALICE_PARTY.to_string()][..];
    create_ping_contract(&ledger_client, &package_id, &application_id, &workflow_id, &command_id, 0).await?;
    let expected_transaction_id = extract_first_transaction_id(&ledger_client).await?;

    let transaction =
        ledger_client.transaction_service().get_flat_transaction_by_id(expected_transaction_id, parties).await?;
    let event = match &transaction.events() {
        [DamlEvent::Created(e)] => e,
        _ => panic!(),
    };
    assert_eq!(&command_id, transaction.command_id());
    assert_eq!(&workflow_id, transaction.workflow_id());
    assert!(!event.event_id().is_empty());
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
    let _lock = initialize_static().await;
    let ledger_client = new_static_sandbox().await?;
    let ledger_end_offset = ledger_client.transaction_service().get_ledger_end().await?;
    assert!(matches!(ledger_end_offset, DamlLedgerOffset::Absolute(_)));
    Ok(())
}

async fn extract_first_created_event(ledger_client: &DamlGrpcClient) -> DamlResult<DamlTransaction> {
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
    let mut flattened_txs: Vec<DamlTransaction> = transactions.into_iter().flatten().collect();
    if flattened_txs.len() == 1 {
        Ok(flattened_txs.swap_remove(0))
    } else {
        Err(DamlError::Other("expected a single transaction".to_string()))
    }
}

async fn extract_first_created_event_id(ledger_client: &DamlGrpcClient) -> DamlResult<String> {
    let first_tx = extract_first_created_event(ledger_client).await?;
    if first_tx.events().len() == 1 {
        let first_event = first_tx.take_events().swap_remove(0);
        match first_event {
            DamlEvent::Created(evt) => Ok(evt.event_id().to_owned()),
            DamlEvent::Archived(_) => Err(DamlError::Other("expected Created event".to_string())),
        }
    } else {
        Err(DamlError::Other("expected a single event".to_string()))
    }
}

async fn extract_first_transaction_id(ledger_client: &DamlGrpcClient) -> DamlResult<String> {
    Ok(extract_first_created_event(ledger_client).await?.transaction_id().to_owned())
}
