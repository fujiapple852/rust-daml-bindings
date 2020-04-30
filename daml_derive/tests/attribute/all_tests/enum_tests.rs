use crate::attribute::test_types::enum_example::{Car, CarContract, SimpleColor};
use crate::common::test_utils::{
    new_static_sandbox, update_create_command_package_id_for_testing, update_exercise_command_package_id_for_testing,
    TestResult, SANDBOX_LOCK,
};
use daml::api::data::event::DamlEvent;
use daml::api::{CommandExecutor, DamlSimpleExecutorBuilder};
use daml::prelude::DamlCommand;
use std::convert::TryInto;

#[tokio::test]
async fn test_using_enum() -> TestResult {
    let _lock = SANDBOX_LOCK.lock()?;
    let client = new_static_sandbox().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();
    let car = Car::new("Alice", "Bob", "Ford", SimpleColor::Green);
    let create_car_command = car.create_command();
    let create_car_command = update_create_command_package_id_for_testing(&client, create_car_command).await?;
    let create_car_result = alice_executor.execute_for_transaction(DamlCommand::Create(create_car_command)).await?;
    let event: DamlEvent = create_car_result.take_events().swap_remove(0);
    let car_contract: CarContract = event.try_created()?.try_into()?;
    assert_eq!(&car, car_contract.data());
    let exercise_command = car_contract.id().repaint_command(SimpleColor::Red);
    let exercise_command = update_exercise_command_package_id_for_testing(&client, exercise_command).await?;
    let repaint_result = alice_executor.execute_for_transaction(DamlCommand::Exercise(exercise_command)).await?;
    let repaint_event: DamlEvent = repaint_result.take_events().swap_remove(1);
    let new_car_contract: CarContract = repaint_event.try_created()?.try_into()?;
    assert_eq!("Alice", new_car_contract.data().owner);
    assert_eq!("Bob", new_car_contract.data().driver);
    assert_eq!("Ford", new_car_contract.data().make);
    assert_eq!(SimpleColor::Red, new_car_contract.data().color);
    Ok(())
}
