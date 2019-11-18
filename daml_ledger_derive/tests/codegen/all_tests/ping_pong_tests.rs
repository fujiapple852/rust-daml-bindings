use crate::common::test_utils::{new_static_sandbox, TestResult, SANDBOX_LOCK};
use daml::prelude::*;
use daml_ledger_api::data::event::DamlEvent;
use daml_ledger_api::DamlSimpleExecutorBuilder;
use daml_ledger_derive::daml_codegen;

daml_codegen!(
    dar_file = r"resources/testing_types_sandbox/archive/TestingTypes-1.0.0-sdk_0.13.34.dar",
    module_filter_regex = "DA.PingPong",
    mode = "Full"
);

#[test]
fn test_create_ping_contract() -> TestResult {
    let _lock = SANDBOX_LOCK.lock()?;
    let client = new_static_sandbox()?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();
    let ping = testing_types_1_0_0::da::ping_pong::Ping::new("Alice", "Bob", 0);
    let create_ping_command = ping.create_command();
    let ping_result = alice_executor.execute_for_transaction_sync(DamlCommand::Create(create_ping_command))?;
    let event: DamlEvent = ping_result.take_events().swap_remove(0);
    let ping_contract: testing_types_1_0_0::da::ping_pong::PingContract = match event {
        DamlEvent::Created(e) => (*e).try_into()?,
        _ => panic!(),
    };
    assert_eq!(&ping, ping_contract.data());
    Ok(())
}
