#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::used_underscore_binding)]

#[macro_use]
extern crate lazy_static;

mod common {
    pub mod test_utils;
}

mod attribute {
    mod test_types {
        pub mod all_data_types;
        pub mod all_map_types;
        pub mod all_optional_types;
        pub mod all_variant_types;
        pub mod enum_example;
        pub mod generic_types;
        pub mod nested_modules;
        pub mod nested_types;
        pub mod pingpong;
        pub mod recursive_types;
        pub mod variant_example;
    }

    mod all_tests {
        mod all_data_types_tests;
        mod enum_tests;
        mod generic_tests;
        mod maps_tests;
        mod nested_modules_tests;
        mod nested_types_tests;
        mod optional_types_tests;
        mod pingpong_tests;
        mod recursive_types_tests;
        mod variant_tests;
    }
}

mod codegen {
    mod all_tests {
        pub mod generic_tests;
        pub mod higher_kinded_tests;
        pub mod nested_datatypes_tests;
        pub mod ping_pong_tests;
        pub mod rent_tests;
    }
}
