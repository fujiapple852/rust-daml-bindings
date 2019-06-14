use crate::common::test_utils::*;
use crate::domain::nested_types::*;
use daml::prelude::*;
use daml_ledger_api::data::event::DamlEvent;

#[test]
pub fn test_round_trip() -> TestResult {
    let mut my_map: DamlTextMap<DamlText> = HashMap::new();
    my_map.insert("test_key".to_owned(), "test value".to_owned());
    let opt_of_list = Some(vec!["text".to_owned()]);
    let list_of_opt_of_map = vec![Some(my_map), None];
    let nested = NestedTypes::new(opt_of_list, list_of_opt_of_map);
    let value: DamlValue = nested.clone().into();
    let nested_again: NestedTypes = value.try_into()?;
    assert_eq!(nested, nested_again);
    Ok(())
}

#[test]
pub fn test_complex_create_and_exercise() -> TestResult {
    let _lock = SANDBOX_LOCK.lock()?;
    let ledger_client = new_static_sandbox()?;

    // construct dummy data
    let mut my_map: DamlTextMap<MyNestedData> = HashMap::new();
    my_map.insert("test_key".to_owned(), MyNestedData::new(true));
    let opt_of_list = Some(vec!["text".to_owned()]);
    let list_of_opt_of_map_of_data = vec![Some(my_map), None];
    let nested_template = NestedTemplate::new("Alice", opt_of_list, list_of_opt_of_map_of_data);

    // submit create command and extract result
    let create_command = nested_template.create();
    let command_result = test_submit_command(&ledger_client, "Alice", create_command)?;
    let event: DamlEvent = command_result.take_events().swap_remove(0);
    let nested_contract: NestedTemplateContract = match event {
        DamlEvent::Created(e) => (*e).try_into()?,
        _ => panic!(),
    };
    assert_eq!(&nested_template, nested_contract.data());

    // construct dummy arguments
    let opt_of_list_param = Some(vec!["foo".to_owned()]);
    let mut my_map_param: DamlTextMap<MyNestedData> = HashMap::new();
    my_map_param.insert("new_test_key_true".to_owned(), MyNestedData::new(true));
    my_map_param.insert("new_test_key_false".to_owned(), MyNestedData::new(false));
    let list_of_opt_of_map_of_data_param = vec![Some(my_map_param), None, None];

    // submit exercise command and extract results
    let exercise_command = nested_contract.pass_complex_arg(opt_of_list_param, list_of_opt_of_map_of_data_param);
    let exercise_result = test_submit_command(&ledger_client, "Alice", exercise_command)?;
    let created_event: DamlEvent = exercise_result.take_events().swap_remove(1);
    let new_contract: NestedTemplateContract = created_event.try_created()?.try_into()?;
    let new_data: &NestedTemplate = new_contract.data();
    assert_eq!(Some(vec!["foo".to_owned()]), new_data.opt_of_list);
    assert_eq!(3, new_data.list_of_opt_of_map_of_data.len());

    Ok(())
}
