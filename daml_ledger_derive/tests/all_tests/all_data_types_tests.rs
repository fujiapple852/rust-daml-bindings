use crate::common::test_utils::*;
use crate::domain::all_data_types::*;
use chrono::{DateTime, Utc};
use daml::prelude::DamlValue;
use std::convert::TryInto;

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
        "1970-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap().date(),
    );
    let value: DamlValue = all.clone().into();
    let all_again: AllDataTypes = value.try_into()?;
    assert_eq!(all, all_again);
    Ok(())
}

#[test]
fn test_scalars_and_lists() -> TestResult {
    let data_list =
        ScalarsAndLists::new("prim", MyData::new(0), vec![(), ()], vec!["prim1".to_owned()], vec![MyData::new(1)]);
    let value: DamlValue = data_list.clone().into();
    let data_list_again: ScalarsAndLists = value.try_into()?;
    assert_eq!(data_list, data_list_again);
    Ok(())
}

#[test]
fn test_all_list_data_types() -> TestResult {
    let all = AllListDataTypes::new(
        vec!["#0:0".to_owned(), "#1:0".to_owned()],
        vec![1, 2, 3],
        vec!["1.23".parse()?, "4.56".parse()?, "7.89".parse()?],
        vec!["some".to_owned(), "text".to_owned(), "list".to_owned()],
        vec!["1970-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap()],
        vec!["Alice".to_owned(), "Bob".to_owned()],
        vec![true, false, true, false],
        vec![(), (), ()],
        vec![
            "1970-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap().date(),
            "1970-01-02T00:00:00Z".parse::<DateTime<Utc>>().unwrap().date(),
        ],
    );
    let value: DamlValue = all.clone().into();
    let all_again: AllListDataTypes = value.try_into()?;
    assert_eq!(all, all_again);
    Ok(())
}
