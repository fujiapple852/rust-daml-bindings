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

#[DamlTemplate(
    package_id = r"510e5612a7970a6d7615bc940e8ee6b4da3eb12f257e59268e729683e9929e8b",
    module_name = "DA.Nested"
)]
pub struct NestedTemplate {
    pub owner: DamlParty,
    pub opt_of_list: DamlOptional<DamlList<DamlText>>,
    pub list_of_opt_of_map_of_data: DamlList<DamlOptional<DamlTextMap<MyNestedData>>>,
}

#[DamlChoices]
impl NestedTemplate {
    #[DoSomethingComplex]
    fn pass_complex_arg(
        &self,
        new_opt_of_list: DamlOptional<DamlList<DamlText>>,
        new_list_of_opt_of_map_of_data: DamlList<DamlOptional<DamlTextMap<MyNestedData>>>,
    ) {
    }
}
