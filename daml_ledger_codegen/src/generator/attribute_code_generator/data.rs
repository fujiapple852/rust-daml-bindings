use syn::{Data, DeriveInput, Fields};

use quote::quote;

use crate::convert::attribute::attr_element::{extract_enum, extract_record, extract_variant, AttrRecord, AttrVariant};
use crate::element::{DamlEnum, DamlRecord, DamlVariant};
use crate::renderer::full::{quote_daml_enum, quote_daml_record, quote_daml_variant};

pub fn generate_data_struct(input: DeriveInput) -> proc_macro::TokenStream {
    let struct_name = input.ident.to_string();
    let tokens = match input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let record: AttrRecord = extract_record(struct_name, fields_named);
                let daml_record = DamlRecord::from(&record);
                quote_daml_record(&daml_record)
            },
            Fields::Unnamed(_) => panic!("tuple struct not supported"),
            Fields::Unit => panic!("unit struct not supported"),
        },
        _ => panic!("the DamlData attribute may only be applied to struct types"),
    };
    let expanded = quote!(
        #tokens
    );
    proc_macro::TokenStream::from(expanded)
}

pub fn generate_data_variant(input: DeriveInput) -> proc_macro::TokenStream {
    let variant_name = input.ident.to_string();
    let tokens = match input.data {
        Data::Enum(data_enum) => {
            let variant: AttrVariant = extract_variant(variant_name, &data_enum);
            let daml_variant = DamlVariant::from(&variant);
            quote_daml_variant(&daml_variant)
        },
        _ => panic!("the DamlVariant attribute may only be applied to enum types"),
    };
    let expanded = quote!(
        #tokens
    );
    proc_macro::TokenStream::from(expanded)
}

pub fn generate_data_enum(input: DeriveInput) -> proc_macro::TokenStream {
    let enum_name = input.ident.to_string();
    let tokens = match input.data {
        Data::Enum(data_enum) => {
            let enum_variants = extract_enum(enum_name, &data_enum);
            let daml_enum = DamlEnum::from(&enum_variants);
            quote_daml_enum(&daml_enum)
        },
        _ => panic!("the DamlEnum attribute may only be applied to enum types"),
    };
    let expanded = quote!(
        #tokens
    );
    proc_macro::TokenStream::from(expanded)
}
