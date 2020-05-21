use crate::common::test_utils::TestResult;
use daml::grpc_api::data::value::DamlValue;
use daml::grpc_api::serialize::{DamlDeserializeInto, DamlSerializeFrom};
use daml_derive::daml_codegen;

daml_codegen!(
    dar_file = r"resources/testing_types_sandbox/archive/TestingTypes-1_0_0-sdk_1_10_0-lf_1_12.dar",
    module_filter_regex = "DA.NestedModuleTest"
);

#[tokio::test]
async fn test_nested_module() -> TestResult {
    use testing_types::da::nested_module_test::parent::child::ChildData;
    use testing_types::da::nested_module_test::parent::ParentData;
    use testing_types::da::nested_module_test::sibling::SiblingData;
    let sibling = SiblingData::new(ParentData::new(ChildData::new("test")));
    let value = DamlValue::serialize_from(sibling.clone());
    let sibling_again: SiblingData = value.deserialize_into()?;
    assert_eq!(sibling, sibling_again);
    Ok(())
}
