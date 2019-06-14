use crate::common::ping_pong::*;
use daml_ledger_api::data::filter::DamlTransactionFilter;
use daml_ledger_api::data::value::DamlRecordField;
use daml_ledger_api::data::value::DamlValue;
use daml_ledger_api::data::DamlActiveContracts;
use daml_ledger_api::service::DamlVerbosity;
use futures::future::Future;
use futures::stream::Stream;

#[test]
fn test_get_active_contracts() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);

    test_create_ping_contract(&ledger_client, &application_id, &workflow_id, &create_test_uuid(COMMAND_ID_PREFIX), 0)?;
    test_create_ping_contract(&ledger_client, &application_id, &workflow_id, &create_test_uuid(COMMAND_ID_PREFIX), 7)?;

    let active_contracts_future = ledger_client.active_contract_service().get_active_contracts(
        DamlTransactionFilter::for_parties(&[ALICE_PARTY, BOB_PARTY][..]),
        DamlVerbosity::Verbose,
    )?;
    let active_contracts: Vec<DamlActiveContracts> = active_contracts_future.collect().wait()?;
    let create_count1: Vec<&DamlRecordField> = active_contracts[0]
        .active_contracts()
        .first()
        .ok_or(ERR_STR)?
        .create_arguments()
        .fields()
        .iter()
        .filter(|rec| match rec.label() {
            Some(label) if label == "count" => true,
            _ => false,
        })
        .collect();
    let create_count2: Vec<&DamlRecordField> = active_contracts[1]
        .active_contracts()
        .first()
        .ok_or(ERR_STR)?
        .create_arguments()
        .fields()
        .iter()
        .filter(|rec| match rec.label() {
            Some(label) if label == "count" => true,
            _ => false,
        })
        .collect();

    let create_count1_val = match *create_count1.first().ok_or(ERR_STR)?.value() {
        DamlValue::Int64(i) => i,
        _ => panic!(),
    };
    let create_count2_val = match *create_count2.first().ok_or(ERR_STR)?.value() {
        DamlValue::Int64(i) => i,
        _ => panic!(),
    };

    assert_eq!(3, active_contracts.len());
    assert_eq!(1, create_count1.len());
    assert_eq!(1, create_count2.len());
    assert_eq!(0, create_count1_val);
    assert_eq!(7, create_count2_val);
    Ok(())
}
