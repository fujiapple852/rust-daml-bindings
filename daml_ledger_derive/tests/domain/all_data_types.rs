#![allow(clippy::too_many_arguments)]
use daml::prelude::*;

#[DamlData]
pub struct MyData {
    pub r: DamlInt64,
}

#[DamlData]
pub struct AllDataTypes {
    pub my_contract_id: DamlContractId,
    pub my_int: DamlInt64,
    pub my_decimal: DamlDecimal,
    pub my_text: DamlText,
    pub my_timestamp: DamlTimestamp,
    pub my_party: DamlParty,
    pub my_bool: DamlBool,
    pub my_unit: DamlUnit,
    pub my_date: DamlDate,
}

#[DamlTemplate(package_id = "test", module_name = "DA.Dummy")]
pub struct AllListDataTypes {
    pub my_contract_ids: DamlList<DamlContractId>,
    pub my_ints: DamlList<DamlInt64>,
    pub my_decimals: DamlList<DamlDecimal>,
    pub my_texts: DamlList<DamlText>,
    pub my_timestamps: DamlList<DamlTimestamp>,
    pub my_parties: DamlList<DamlParty>,
    pub my_bools: DamlList<DamlBool>,
    pub my_units: DamlList<DamlUnit>,
    pub my_dates: DamlList<DamlDate>,
}

#[DamlChoices]
impl AllListDataTypes {
    #[SomeChoice]
    pub fn func_with_all_scalar_list_params(
        my_contract_ids: DamlList<DamlContractId>,
        my_ints: DamlList<DamlInt64>,
        my_decimals: DamlList<DamlDecimal>,
        my_texts: DamlList<DamlText>,
        my_timestamps: DamlList<DamlTimestamp>,
        my_parties: DamlList<DamlParty>,
        my_bools: DamlList<DamlBool>,
        my_units: DamlList<DamlUnit>,
        my_dates: DamlList<DamlDate>,
    ) {
    }
}

#[DamlTemplate(package_id = "test", module_name = "DA.Dummy")]
pub struct ScalarsAndLists {
    pub prim: DamlParty,
    pub nested: MyData,
    pub list_unit: DamlList<DamlUnit>,
    pub list_prim: DamlList<DamlParty>,
    pub list_nested: DamlList<MyData>,
}

#[DamlChoices]
impl ScalarsAndLists {
    #[SomeChoice]
    pub fn func_with_list_param(
        prim: DamlParty,
        nested: MyData,
        list_unit: DamlList<DamlUnit>,
        list_prim: DamlList<DamlParty>,
        list_nested: DamlList<MyData>,
    ) {
    }
}
