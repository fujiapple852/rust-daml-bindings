use crate::common::ping_pong::{
    create_ping_contract, create_test_uuid, new_static_sandbox, TestResult, ALICE_PARTY, APPLICATION_ID_PREFIX,
    BOB_PARTY, COMMAND_ID_PREFIX, ERR_STR, PINGPONG_MODULE_NAME, STATIC_SANDBOX_LOCK, WORKFLOW_ID_PREFIX,
};

use daml::util::package::find_module_package_id;
use daml_grpc::data::filter::DamlTransactionFilter;
use daml_grpc::data::value::{DamlRecordField, DamlValue};
use daml_grpc::data::{DamlActiveContracts, DamlResult};
use daml_grpc::service::DamlVerbosity;
use futures::StreamExt;

#[tokio::test]
async fn test_get_active_contracts() -> TestResult {
    let _lock = STATIC_SANDBOX_LOCK.lock();
    let ledger_client = new_static_sandbox().await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    let application_id = create_test_uuid(APPLICATION_ID_PREFIX);
    let workflow_id = create_test_uuid(WORKFLOW_ID_PREFIX);
    create_ping_contract(
        &ledger_client,
        &package_id,
        &application_id,
        &workflow_id,
        &create_test_uuid(COMMAND_ID_PREFIX),
        0,
    )
    .await?;
    create_ping_contract(
        &ledger_client,
        &package_id,
        &application_id,
        &workflow_id,
        &create_test_uuid(COMMAND_ID_PREFIX),
        7,
    )
    .await?;
    let active_contracts_future = ledger_client
        .active_contract_service()
        .get_active_contracts(DamlTransactionFilter::for_parties(&[ALICE_PARTY, BOB_PARTY][..]), DamlVerbosity::Verbose)
        .await?;
    let active_contracts: Vec<DamlResult<DamlActiveContracts>> = active_contracts_future.collect().await;
    let active_contracts: Vec<_> = active_contracts.into_iter().map(std::result::Result::unwrap).collect();
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
