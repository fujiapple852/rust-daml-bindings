use crate::common::test_utils::TestResult;

#[test]
pub fn test_nested_modules() -> TestResult {
    use crate::attribute::test_types::nested_modules::da::my_module::my_sub_module::MyOuterData;
    use crate::attribute::test_types::nested_modules::da::my_module::MyData;
    let data = MyData::new("Alice");
    let outer = MyOuterData::new("Bob", data, vec![MyData::new("Bob1"), MyData::new("Bob2")]);
    assert_eq!("Bob", outer.name);
    assert_eq!("Alice", outer.data.name);
    assert_eq!(vec!["Bob1", "Bob2"], outer.data_list.iter().map(|d| &d.name).collect::<Vec<_>>());
    Ok(())
}
