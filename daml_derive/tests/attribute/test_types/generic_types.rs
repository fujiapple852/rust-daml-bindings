pub mod da {
    use daml::prelude::*;
    #[DamlData]
    pub struct SimpleRecord {
        pub data: DamlInt64,
    }
    pub mod my_module {
        use daml::prelude::*;
        #[DamlData]
        pub struct GenericRecord<T, R> {
            pub t: T,
            pub r: R,
        }
        #[DamlData]
        pub struct ConcreteRecord {
            pub val: crate::attribute::test_types::generic_types::da::my_module::GenericRecord<
                DamlText,
                DamlList<crate::attribute::test_types::generic_types::da::SimpleRecord>,
            >,
        }
    }
}
