use crate::attribute::test_types::all_data_types::{AllDataTypes, AllListDataTypes, MyData, ScalarsAndLists};
use crate::common::test_utils::TestResult;
use chrono::{DateTime, NaiveDate, Utc};
use daml::grpc_api::data::value::DamlValue;
use daml::grpc_api::primitive_types::{DamlContractId, DamlParty};
use daml::grpc_api::serialize::{DamlDeserializeInto, DamlSerializeInto};

#[test]
fn test_all_data_types() -> TestResult {
    let all = AllDataTypes::new(
        "#0:0",
        23,
        23.1,
        "hello!",
        "1970-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        "Alice",
        false,
        (),
        "1970-01-01".parse::<NaiveDate>().unwrap(),
    );
    let value: DamlValue = all.clone().serialize_into();
    let all_again: AllDataTypes = value.deserialize_into()?;
    assert_eq!(all, all_again);
    Ok(())
}

#[test]
fn test_scalars_and_lists() -> TestResult {
    let data_list =
        ScalarsAndLists::new("prim", MyData::new(0), vec![(), ()], vec![DamlParty::new("prim1")], vec![MyData::new(1)]);
    let value: DamlValue = data_list.clone().serialize_into();
    let data_list_again: ScalarsAndLists = value.deserialize_into()?;
    assert_eq!(data_list, data_list_again);
    Ok(())
}

#[test]
fn test_all_list_data_types() -> TestResult {
    let all = AllListDataTypes::new(
        vec![DamlContractId::new("#0:0"), DamlContractId::new("#1:0")],
        vec![1, 2, 3],
        vec!["1.23".parse()?, "4.56".parse()?, "7.89".parse()?],
        vec!["some".to_owned(), "text".to_owned(), "list".to_owned()],
        vec!["1970-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap()],
        vec![DamlParty::new("Alice"), DamlParty::new("Bob")],
        vec![true, false, true, false],
        vec![(), (), ()],
        vec!["1970-01-01".parse::<NaiveDate>().unwrap(), "1970-01-02".parse::<NaiveDate>().unwrap()],
    );
    let value: DamlValue = all.clone().serialize_into();
    let all_again: AllListDataTypes = value.deserialize_into()?;
    assert_eq!(all, all_again);
    Ok(())
}
