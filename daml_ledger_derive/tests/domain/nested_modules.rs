pub mod da {
    pub mod my_module {
        use daml::prelude::*;

        #[DamlData]
        pub struct MyData {
            pub name: DamlParty,
        }

        pub mod my_sub_module {
            use daml::prelude::*;

            #[DamlData]
            pub struct MyOuterData {
                pub name: DamlParty,
                pub data: crate::domain::nested_modules::da::my_module::MyData,
                pub data_list: DamlList<crate::domain::nested_modules::da::my_module::MyData>,
            }
        }
    }
}
