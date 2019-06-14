#![warn(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate lazy_static;

mod common {
    pub mod test_utils;
}

mod domain {
    pub mod all_data_types;
    pub mod all_map_types;
    pub mod all_optional_types;
    pub mod all_variant_types;
    pub mod nested_modules;
    pub mod nested_types;
    pub mod pingpong;
    pub mod variant_example;
}

mod all_tests {
    mod all_data_types_tests;
    mod maps_tests;
    mod nested_modules_tests;
    mod nested_types_tests;
    mod optional_types_tests;
    mod pingpong_tests;
    mod variant_tests;
}
