use daml::prelude::*;

#[DamlData]
pub struct MyNestedData {
    pub my_bool: DamlBool,
}

#[DamlData]
pub struct NestedTypes {
    pub opt_of_list: DamlOptional<DamlList<DamlText>>,
    pub list_of_opt_of_map: DamlList<DamlOptional<DamlTextMap<DamlText>>>,
}

#[DamlTemplate(package_id = r"test", module_name = "Fuji.Nested")]
pub struct NestedTemplate {
    pub owner: DamlParty,
    pub opt_of_list: DamlOptional<DamlList<DamlText>>,
    pub list_of_opt_of_map_of_data: DamlList<DamlOptional<DamlTextMap<MyNestedData>>>,
    pub map_of_data_to_text: DamlGenMap<MyNestedData, DamlText>,
}

#[DamlChoices]
impl NestedTemplate {
    #[DoSomethingComplex]
    fn do_something_complex(
        &self,
        new_opt_of_list: DamlOptional<DamlList<DamlText>>,
        new_list_of_opt_of_map_of_data: DamlList<DamlOptional<DamlTextMap<MyNestedData>>>,
        new_map_of_data_to_text: DamlGenMap<MyNestedData, DamlText>,
    ) {
    }
}
