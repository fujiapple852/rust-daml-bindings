use crate::attribute::test_types::all_variant_types::*;
use crate::attribute::test_types::variant_example::*;
use crate::common::test_utils::*;
use daml::prelude::DamlParty;
use daml_ledger_api::data::command::DamlCommand;
use daml_ledger_api::data::event::DamlEvent;
use daml_ledger_api::{CommandExecutor, DamlSimpleExecutorBuilder};
use std::collections::HashMap;
use std::convert::TryInto;

#[tokio::test]
async fn test_circle_variant() -> TestResult {
    let _lock = SANDBOX_LOCK.lock()?;
    let client = new_static_sandbox().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();

    let circle_template =
        CircleTemplate::new("Alice", Circle::new(1.223_000_000_1, Color::Other(RGBA::new(255, 0, 100, 99))));
    let create_command = circle_template.create_command();
    let create_command = update_create_command_package_id_for_testing(&client, create_command).await?;
    let transaction = alice_executor.execute_for_transaction(DamlCommand::Create(create_command)).await?;
    let event: DamlEvent = transaction.take_events().swap_remove(0);
    let circle_contract: CircleTemplateContract = event.try_created()?.try_into()?;
    assert_eq!(&circle_template, circle_contract.data());
    let exercise_command =
        circle_contract.id().replace_circle_command(Circle::new(1.223_000_000_1, Color::Custom(vec![1, 2, 3])));
    let exercise_command = update_exercise_command_package_id_for_testing(&client, exercise_command).await?;
    let exercise_transaction = alice_executor.execute_for_transaction(DamlCommand::Exercise(exercise_command)).await?;
    let created_event: DamlEvent = exercise_transaction.take_events().swap_remove(1);
    let new_circle_contract: CircleTemplateContract = created_event.try_created()?.try_into()?;
    if let Color::Custom(values) = &new_circle_contract.data().circle.color {
        assert_eq!(&vec![1, 2, 3], values);
    } else {
        panic!("expected Custom Color")
    }
    Ok(())
}

#[tokio::test]
async fn test_all_variant_types() -> TestResult {
    let _lock = SANDBOX_LOCK.lock()?;
    let client = new_static_sandbox().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();

    let mut map_of_party: HashMap<String, DamlParty> = HashMap::new();
    map_of_party.insert("sender".to_owned(), "Alice".to_owned());
    map_of_party.insert("receiver".to_owned(), "Bob".to_owned());
    let mut map_of_records: HashMap<String, RecordArgument> = HashMap::new();
    map_of_records.insert("Alice".to_owned(), RecordArgument::new(8, vec!["test1".to_owned()]));
    map_of_records.insert("Bob".to_owned(), RecordArgument::new(4, vec!["test2".to_owned()]));
    let variants = vec![
        AllVariantTypes::NoArgument,
        AllVariantTypes::TupleStructPrimitive("foobar".to_owned()),
        AllVariantTypes::TupleStructListOfPrimitive(vec![1, 2, 3]),
        AllVariantTypes::TupleStructListOfRecord(vec![RecordArgument::new(1, vec!["test".to_owned()])]),
        AllVariantTypes::TupleStructMapOfPrimitive(map_of_party),
        AllVariantTypes::TupleStructMapOfRecord(map_of_records),
        AllVariantTypes::TupleStructOptionalOfPrimitive(None),
        AllVariantTypes::TupleStructOptionalOfPrimitive(Some(true)),
        AllVariantTypes::TupleStructOptionalOfRecord(None),
        AllVariantTypes::TupleStructOptionalOfRecord(Some(RecordArgument::new(1, vec!["test".to_owned()]))),
        AllVariantTypes::TupleStructComplexType(Some(vec![1, 2, 3])),
        AllVariantTypes::TupleStructRecord(RecordArgument::new(1, vec!["abcd".to_owned()])),
        AllVariantTypes::Record(AnonRecord::new(1, vec!["abcd".to_owned()])),
    ];
    let variant_template = VariantTemplate::new("Alice", variants);
    let create_command = variant_template.create_command();
    let create_command = update_create_command_package_id_for_testing(&client, create_command).await?;
    let transaction = alice_executor.execute_for_transaction(DamlCommand::Create(create_command)).await?;
    let event: DamlEvent = transaction.take_events().swap_remove(0);
    let variant_contract: VariantTemplateContract = event.try_created()?.try_into()?;
    assert_eq!(&variant_template, variant_contract.data());
    Ok(())
}
