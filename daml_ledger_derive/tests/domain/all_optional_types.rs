use daml::prelude::*;

#[DamlData]
pub struct MyData {
    pub r: DamlInt64,
}

#[DamlData]
pub struct DataWithOptional {
    opt_unit: DamlOptional<DamlUnit>,
    opt_primitive: DamlOptional<DamlInt64>,
    opt_party: DamlOptional<DamlParty>,
    opt_record: DamlOptional<MyData>,
    opt_list: DamlOptional<DamlList<DamlParty>>,
    opt_map: DamlOptional<DamlTextMap<DamlInt64>>,
}
