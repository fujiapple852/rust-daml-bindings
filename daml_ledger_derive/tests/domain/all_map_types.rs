use daml::prelude::*;
use std::collections::HashMap;

#[DamlData]
pub struct MyRecord {
    field_aaa: DamlInt64,
    field_bbb: DamlList<DamlText>,
}

#[DamlData]
pub struct DataWithMap {
    pub map_of_unit: DamlTextMap<DamlUnit>,
    pub map_of_primitive: DamlTextMap<DamlInt64>,
    pub map_of_records: DamlTextMap<MyRecord>,
    pub map_of_lists: DamlTextMap<DamlList<DamlInt64>>,
}
