use crate::common::test_utils::{
    new_static_sandbox, update_create_command_package_id_for_testing, TestResult, SANDBOX_LOCK,
};
use daml::grpc_api::data::command::DamlCommand;
use daml::grpc_api::data::event::DamlEvent;
use daml::grpc_api::data::value::DamlValue;
use daml::grpc_api::primitive_types::DamlText;
use daml::grpc_api::serialize::{DamlDeserializeInto, DamlSerializeFrom};
use daml::grpc_api::CommandExecutor;
use daml::grpc_api::DamlSimpleExecutorBuilder;
use daml_derive::daml_codegen;
use std::convert::TryInto;

daml_codegen!(
    dar_file = r"resources/testing_types_sandbox/archive/TestingTypes-1_0_0-sdk_1_4_0-lf_1_8.dar",
    module_filter_regex = "DA.GenericTypes"
);

#[test]
fn test_generic_local_roundtrip() -> TestResult {
    use testing_types::da::generic_types::{ConcreteDataRecord, GenericDataRecord};
    let conc = ConcreteDataRecord::new(GenericDataRecord::new(Some(vec![0]), vec!["".to_string()], 1));
    let value = DamlValue::serialize_from(conc.clone());
    let conc_again: ConcreteDataRecord = value.deserialize_into()?;
    assert_eq!(conc, conc_again);
    Ok(())
}

#[test]
fn test_partial_generic_local_roundtrip() -> TestResult {
    use testing_types::da::generic_types::{GenericDataRecord, PartialConcreteDataRecord};
    let conc = PartialConcreteDataRecord::<DamlText>::new(GenericDataRecord::new(Some(vec![0]), "".to_string(), 1));
    let value = DamlValue::serialize_from(conc.clone());
    let conc_again: PartialConcreteDataRecord<DamlText> = value.deserialize_into()?;
    assert_eq!(conc, conc_again);
    Ok(())
}

#[test]
fn test_recursive_generic_record_local_roundtrip() -> TestResult {
    use testing_types::da::generic_types::{GenericWrapperRecord, PatternRecord};
    let pattern = PatternRecord::new(GenericWrapperRecord::new(PatternRecord::new(Some(GenericWrapperRecord::new(
        PatternRecord::new(None),
    )))));
    let value = DamlValue::serialize_from(pattern.clone());
    let pattern_again = value.deserialize_into()?;
    assert_eq!(pattern, pattern_again);
    Ok(())
}

#[test]
fn test_recursive_generic_variant_local_roundtrip() -> TestResult {
    use testing_types::da::generic_types::{GenericWrapperRecord, PatternVariant};
    let pattern = PatternVariant::PStart(GenericWrapperRecord::new(PatternVariant::PEnd));
    let value = DamlValue::serialize_from(pattern.clone());
    let pattern_again = value.deserialize_into()?;
    assert_eq!(pattern, pattern_again);
    Ok(())
}

#[tokio::test]
async fn test_create_contract_with_generic() -> TestResult {
    use testing_types::da::generic_types::{
        ConcreteDataRecord, GenericDataRecord, GenericWrapperRecord, PatternVariant, TemplateWithGeneric,
        TemplateWithGenericContract,
    };
    let _lock = SANDBOX_LOCK.lock().await;
    let client = new_static_sandbox().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();
    let template_with_generic = TemplateWithGeneric::new(
        "Alice",
        GenericDataRecord::new(Some(vec![0, 1, 2]), vec!["first".to_owned(), "second".to_owned()], 10),
        GenericDataRecord::new(101, "middle".to_string(), 30),
        ConcreteDataRecord::new(GenericDataRecord::new(Some(vec![5, 4, 3]), vec!["single".to_string()], 30)),
        PatternVariant::PStart(GenericWrapperRecord::new(PatternVariant::PEnd)),
    );
    let create_command = template_with_generic.create_command();
    let create_command = update_create_command_package_id_for_testing(&client, create_command).await?;
    let create_result = alice_executor.execute_for_transaction(DamlCommand::Create(create_command)).await?;

    let event: DamlEvent = create_result.take_events().swap_remove(0);
    let contract: TemplateWithGenericContract = match event {
        DamlEvent::Created(e) => (*e).try_into()?,
        DamlEvent::Archived(_) => panic!(),
    };
    assert_eq!(&template_with_generic, contract.data());
    Ok(())
}
