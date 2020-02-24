use crate::attribute::test_types::generic_types::da::SimpleRecord;
use crate::common::test_utils::TestResult;

#[test]
pub fn test_generic_struct() -> TestResult {
    use crate::attribute::test_types::generic_types::da::my_module::{ConcreteRecord, GenericRecord};
    let data = ConcreteRecord::new(GenericRecord::new("some text".to_owned(), vec![SimpleRecord::new(101)]));
    assert_eq!("some text", data.val.t);
    assert_eq!(101, data.val.r[0].data);
    Ok(())
}
