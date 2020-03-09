pub mod error;

pub mod archive {
    pub mod wrapper {
        mod payload {
            mod archive_payload;
            mod data_payload;
            mod field_payload;
            mod interned;
            mod module_payload;
            mod package_payload;
            mod type_payload;
            mod typevar_payload;
            mod util;

            pub use archive_payload::*;
            pub use data_payload::*;
            pub use field_payload::*;
            pub use interned::*;
            pub use module_payload::*;
            pub use package_payload::*;
            pub use type_payload::*;
            pub use typevar_payload::*;
        }

        mod archive_wrapper;
        mod data_data_box_checker;
        mod data_wrapper;
        mod field_wrapper;
        mod module_wrapper;
        mod package_wrapper;
        mod type_wrapper;
        mod typevar_wrapper;

        pub use archive_wrapper::*;
        pub use data_data_box_checker::*;
        pub use data_wrapper::*;
        pub use field_wrapper::*;
        pub use module_wrapper::*;
        pub use package_wrapper::*;
        pub use type_wrapper::*;
        pub use typevar_wrapper::*;

        // TODO, leaky abstractions
        pub use payload::DamlArchivePayload;
        pub use payload::DamlKindPayload;
        pub use payload::DamlTypePayload;
        pub use payload::DamlVarPayload;
        pub use payload::PackageInternedResolver;
    }

    pub mod archive_converter;

    // TODO, leaky abstraction
    pub use wrapper::DamlArchivePayload;
}

pub mod attribute {
    pub mod attr_element {
        pub use attr_choice::*;
        pub use attr_data::*;
        pub use attr_field::*;
        pub use attr_type::*;
        pub use attr_variant::*;

        mod attr_choice;
        mod attr_data;
        mod attr_field;
        mod attr_type;
        mod attr_variant;
    }

    pub mod attribute_converter;
}
