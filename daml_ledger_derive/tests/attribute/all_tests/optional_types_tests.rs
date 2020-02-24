use crate::attribute::test_types::all_optional_types::*;
use crate::common::test_utils::TestResult;
use daml::prelude::{DamlDeserializeInto, DamlParty, DamlSerializeInto};
use daml_ledger_api::data::value::DamlValue;

#[test]
pub fn test_optionals() -> TestResult {
    let data_with_optional = DataWithOptional::new(
        Some(()),
        None,
        Some(DamlParty::new("Alice")),
        Some(MyData::new(99)),
        Some(vec![DamlParty::new("Alice")]),
        None,
    );
    let value: DamlValue = data_with_optional.clone().serialize_into();
    let data_with_optional_again = value.deserialize_into()?;
    assert_eq!(data_with_optional, data_with_optional_again);
    Ok(())
}
