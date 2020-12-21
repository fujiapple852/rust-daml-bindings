use crate::common::test_utils::TestResult;
use daml::grpc_api::data::value::DamlValue;
use daml::grpc_api::serialize::{DamlDeserializeInto, DamlSerializeFrom};
use daml_derive::daml_codegen;

daml_codegen!(
    dar_file = r"resources/testing_types_sandbox/archive/TestingTypes-1_0_0-sdk_1_8_0-lf_1_8.dar",
    module_filter_regex = "DA.HigherKindTest"
);

#[test]
fn test_higher_kinded() -> TestResult {
    use testing_types::da::higher_kind_test::{DataWithHigherKindField, MyData};

    // This tests that the field `hktField` is omitted from `DataWithHigherKindField`.
    let data_with_hkt: DataWithHigherKindField = DataWithHigherKindField::new(MyData::new("test"));
    let value = DamlValue::serialize_from(data_with_hkt.clone());
    let overrides_again: DataWithHigherKindField = value.deserialize_into()?;
    assert_eq!(data_with_hkt, overrides_again);
    Ok(())
}
