use crate::attribute::test_types::all_map_types::{DataWithMap, MyRecord};
use crate::common::test_utils::TestResult;
use daml::grpc_api::data::value::DamlValue;
use daml::grpc_api::primitive_types::{DamlGenMap, DamlInt64, DamlList, DamlText, DamlUnit};
use daml::grpc_api::serialize::{DamlDeserializeInto, DamlSerializeInto};
use daml::prelude::DamlTextMap;

#[test]
pub fn test_maps() -> TestResult {
    let mut map_of_unit: DamlTextMap<DamlUnit> = DamlTextMap::new();
    map_of_unit.insert("Alice".to_owned(), ());
    map_of_unit.insert("Bob".to_owned(), ());
    map_of_unit.insert("John".to_owned(), ());
    let mut map_of_primitive: DamlTextMap<DamlInt64> = DamlTextMap::new();
    map_of_primitive.insert("Alice".to_owned(), 1);
    map_of_primitive.insert("Bob".to_owned(), 2);
    map_of_primitive.insert("John".to_owned(), 3);
    let mut map_of_records: DamlTextMap<MyRecord> = DamlTextMap::new();
    map_of_records.insert("Alice".to_owned(), MyRecord::new(10, vec!["Alice text".to_owned()]));
    map_of_records.insert("Bob".to_owned(), MyRecord::new(20, vec!["Bob text".to_owned()]));
    let mut map_of_list: DamlTextMap<DamlList<DamlInt64>> = DamlTextMap::new();
    map_of_list.insert("a".to_owned(), vec![1, 2, 3]);
    map_of_list.insert("b".to_owned(), vec![9, 8, 7]);
    let mut map_of_int_to_text: DamlGenMap<DamlInt64, DamlText> = DamlGenMap::new();
    map_of_int_to_text.insert(100, "a".to_owned());
    map_of_int_to_text.insert(200, "b".to_owned());
    let data_with_map =
        DataWithMap::new(map_of_unit, map_of_primitive, map_of_records, map_of_list, map_of_int_to_text);
    let value: DamlValue = data_with_map.clone().serialize_into();
    let data_with_map_again: DataWithMap = value.deserialize_into()?;
    assert_eq!(data_with_map, data_with_map_again);
    Ok(())
}
