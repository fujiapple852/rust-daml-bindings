use crate::attribute::test_types::all_optional_types::*;
use crate::common::test_utils::TestResult;
use daml_ledger_api::data::value::DamlValue;
use std::convert::TryInto;

#[test]
pub fn test_optionals() -> TestResult {
    let data_with_optional = DataWithOptional::new(
        Some(()),
        None,
        Some("Alice".to_owned()),
        Some(MyData::new(99)),
        Some(vec!["Alice".to_owned()]),
        None,
    );
    let value: DamlValue = data_with_optional.clone().into();
    let data_with_optional_again = value.try_into()?;
    assert_eq!(data_with_optional, data_with_optional_again);
    Ok(())
}
