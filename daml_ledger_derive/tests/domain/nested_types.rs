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
    package_id = r"045a30fb3e25804277456215a9bd7b8d93406e62a87ac1c07f6aeb7c9e1ca066",
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
