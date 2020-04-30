use crate::attribute::test_types::pingpong::{Ping, PingContract, UserData};
use crate::common::test_utils::{
    new_static_sandbox, update_create_command_package_id_for_testing, update_exercise_command_package_id_for_testing,
    TestResult, SANDBOX_LOCK,
};
use daml::api::data::command::DamlCommand;
use daml::api::data::event::DamlEvent;
use daml::api::data::value::{DamlRecord, DamlValue};
use daml::api::data::DamlIdentifier;
use daml::api::{CommandExecutor, DamlSimpleExecutorBuilder};
use daml::macros::daml_path;
use daml::prelude::{DamlDeserializeInto, DamlSerializeInto};
use std::convert::TryInto;

#[test]
fn test_local_round_trip() -> TestResult {
    let ping = Ping::new("Alice", "Bob", 0);
    let expected_id = DamlIdentifier::new("omitted", "DA.PingPong", "Ping");
    assert_eq!(&expected_id.module_name(), &Ping::package_id().module_name());
    assert_eq!(&expected_id.entity_name(), &Ping::package_id().entity_name());
    let ping_value: DamlValue = ping.serialize_into();
    assert_eq!("Alice", ping_value.extract(daml_path!(sender#p))?);
    assert_eq!("Bob", ping_value.extract(daml_path!(receiver#p))?);
    assert_eq!(&0, ping_value.extract(daml_path!(count#i))?);
    let ping_again: Ping = ping_value.deserialize_into()?;
    assert_eq!("Alice", ping_again.sender);
    assert_eq!("Bob", ping_again.receiver);
    assert_eq!(0, ping_again.count);
    Ok(())
}

#[tokio::test]
async fn test_ledger_create() -> TestResult {
    let _lock = SANDBOX_LOCK.lock()?;
    let client = new_static_sandbox().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();
    let ping = Ping::new("Alice", "Bob", 0);
    let create_ping_command = ping.create_command();
    let create_ping_command = update_create_command_package_id_for_testing(&client, create_ping_command).await?;
    let ping_result = alice_executor.execute_for_transaction(DamlCommand::Create(create_ping_command)).await?;
    let event: DamlEvent = ping_result.take_events().swap_remove(0);
    let ping_contract: PingContract = match event {
        DamlEvent::Created(e) => (*e).try_into()?,
        _ => panic!(),
    };
    assert_eq!(&ping, ping_contract.data());
    Ok(())
}

#[tokio::test]
async fn test_ledger_create_and_exercise_with_nested() -> TestResult {
    let _lock = SANDBOX_LOCK.lock()?;
    let client = new_static_sandbox().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();

    let ping = Ping::new("Alice", "Bob", 0);
    let create_ping_command = ping.create_command();
    let create_ping_command = update_create_command_package_id_for_testing(&client, create_ping_command).await?;
    let ping_result = alice_executor.execute_for_transaction(DamlCommand::Create(create_ping_command)).await?;

    // extract the CreatedEvent we got back and use this to build a PingContract
    let event: DamlEvent = ping_result.take_events().swap_remove(0);
    let ping_contract: PingContract = event.try_created()?.try_into()?;
    assert_eq!(&ping, ping_contract.data());

    // Generate an ExerciseCommand from the PingContract (reset ResetPingCount)
    let exercise_command = ping_contract.id().from_user_data_command(5, UserData::new("foo", 2));
    let exercise_command = update_exercise_command_package_id_for_testing(&client, exercise_command).await?;
    let ping_reset_result = alice_executor.execute_for_transaction(DamlCommand::Exercise(exercise_command)).await?;

    // extract the CreatedEvent we got back (item 1 as 0 is the archive event of our initial Ping contract) and use this
    // to build a PingContract
    let ping_reset_event: DamlEvent = ping_reset_result.take_events().swap_remove(1);
    let ping_reset_contract: PingContract = ping_reset_event.try_created()?.try_into()?;

    assert_eq!(7, ping_reset_contract.data().count);
    Ok(())
}
