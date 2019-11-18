use crate::attribute::test_types::recursive_types::*;
use crate::common::test_utils::TestResult;
use daml_ledger_api::data::value::DamlValue;
use std::convert::TryInto;

#[test]
pub fn test_recursive_type() -> TestResult {
    let inner = InnerType::new(
        "The Inner",
        Some(Box::new(BoxedOuterType::new("The Boxed Outer", InnerType::new("The Inner 2", None, None)))),
        OuterType::new("The Outer", InnerType::new("The Inner 3", None, None)),
    );
    let value: DamlValue = inner.clone().into();
    let inner_again = value.try_into()?;
    assert_eq!(inner, inner_again);
    Ok(())
}
