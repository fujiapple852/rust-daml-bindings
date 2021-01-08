use crate::attribute::test_types::nested_types::{MyNestedData, NestedTemplate, NestedTemplateContract, NestedTypes};
use crate::common::test_utils::{
    new_static_sandbox, update_create_command_package_id_for_testing, update_exercise_command_package_id_for_testing,
    TestResult, SANDBOX_LOCK,
};
use daml::grpc_api::data::command::DamlCommand;
use daml::grpc_api::data::event::DamlEvent;
use daml::grpc_api::data::value::DamlValue;
use daml::grpc_api::primitive_types::{DamlText, DamlTextMap};
use daml::grpc_api::serialize::{DamlDeserializeInto, DamlSerializeInto};
use daml::grpc_api::CommandExecutor;
use daml::grpc_api::DamlSimpleExecutorBuilder;
use std::collections::HashMap;
use std::convert::TryInto;

#[test]
pub fn test_round_trip() -> TestResult {
    let mut my_map: DamlTextMap<DamlText> = HashMap::new();
    my_map.insert("test_key".to_owned(), "test value".to_owned());
    let opt_of_list = Some(vec!["text".to_owned()]);
    let list_of_opt_of_map = vec![Some(my_map), None];
    let nested = NestedTypes::new(opt_of_list, list_of_opt_of_map);
    let value: DamlValue = nested.clone().serialize_into();
    let nested_again: NestedTypes = value.deserialize_into()?;
    assert_eq!(nested, nested_again);
    Ok(())
}

#[tokio::test]
pub async fn test_complex_create_and_exercise() -> TestResult {
    let _lock = SANDBOX_LOCK.lock().await;
    let client = new_static_sandbox().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client).act_as("Alice").build()?;

    // construct dummy data
    let mut my_map: DamlTextMap<MyNestedData> = HashMap::new();
    my_map.insert("test_key".to_owned(), MyNestedData::new(true));
    let opt_of_list = Some(vec!["text".to_owned()]);
    let list_of_opt_of_map_of_data = vec![Some(my_map), None];
    let nested_template = NestedTemplate::new("Alice", opt_of_list, list_of_opt_of_map_of_data);

    // submit create command and extract result
    let create_command = nested_template.create_command();
    let create_command = update_create_command_package_id_for_testing(&client, create_command).await?;
    let command_result = alice_executor.execute_for_transaction(DamlCommand::Create(create_command)).await?;
    let event: DamlEvent = command_result.take_events().swap_remove(0);
    let nested_contract: NestedTemplateContract = match event {
        DamlEvent::Created(e) => (*e).try_into()?,
        DamlEvent::Archived(_) => panic!(),
    };
    assert_eq!(&nested_template, nested_contract.data());

    // construct dummy arguments
    let opt_of_list_param = Some(vec!["foo".to_owned()]);
    let mut my_map_param: DamlTextMap<MyNestedData> = HashMap::new();
    my_map_param.insert("new_test_key_true".to_owned(), MyNestedData::new(true));
    my_map_param.insert("new_test_key_false".to_owned(), MyNestedData::new(false));
    let list_of_opt_of_map_of_data_param = vec![Some(my_map_param), None, None];

    // submit exercise command and extract results
    let exercise_command =
        nested_contract.id().do_something_complex_command(opt_of_list_param, list_of_opt_of_map_of_data_param);
    let exercise_command = update_exercise_command_package_id_for_testing(&client, exercise_command).await?;
    let exercise_result = alice_executor.execute_for_transaction(DamlCommand::Exercise(exercise_command)).await?;
    let created_event: DamlEvent = exercise_result.take_events().swap_remove(1);
    let new_contract: NestedTemplateContract = created_event.try_created()?.try_into()?;
    let new_data: &NestedTemplate = new_contract.data();
    assert_eq!(Some(vec!["foo".to_owned()]), new_data.opt_of_list);
    assert_eq!(3, new_data.list_of_opt_of_map_of_data.len());

    Ok(())
}
