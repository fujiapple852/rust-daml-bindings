use crate::common::test_utils::{
    new_static_sandbox, update_create_command_package_id_for_testing, TestResult, SANDBOX_LOCK,
};
use daml::grpc_api::data::command::DamlCommand;
use daml::grpc_api::data::event::DamlEvent;
use daml::grpc_api::CommandExecutor;
use daml::grpc_api::DamlSimpleExecutorBuilder;
use daml_derive::daml_codegen;
use std::convert::TryInto;

daml_codegen!(
    dar_file = r"resources/testing_types_sandbox/archive/TestingTypes-1_0_0-sdk_1_7_0-lf_1_8.dar",
    module_filter_regex = "DA.PingPong",
    mode = "Full"
);

#[tokio::test]
async fn test_create_ping_contract() -> TestResult {
    let _lock = SANDBOX_LOCK.lock().await;
    let client = new_static_sandbox().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();
    let ping = testing_types::da::ping_pong::Ping::new("Alice", "Bob", 0);
    let create_ping_command = ping.create_command();
    let create_ping_command = update_create_command_package_id_for_testing(&client, create_ping_command).await?;
    let ping_result = alice_executor.execute_for_transaction(DamlCommand::Create(create_ping_command)).await?;
    let event: DamlEvent = ping_result.take_events().swap_remove(0);
    let ping_contract: testing_types::da::ping_pong::PingContract = match event {
        DamlEvent::Created(e) => (*e).try_into()?,
        DamlEvent::Archived(_) => panic!(),
    };
    assert_eq!(&ping, ping_contract.data());
    Ok(())
}
