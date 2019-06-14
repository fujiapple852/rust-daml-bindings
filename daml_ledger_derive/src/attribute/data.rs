use crate::quote::{quote_enum_and_impl, quote_struct_and_impl};
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn generate_data(input: DeriveInput) -> proc_macro::TokenStream {
    let struct_name = input.ident.to_string();
    let tokens = match input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => quote_struct_and_impl(&struct_name, fields_named),
            Fields::Unnamed(_) => panic!("tuple struct not supported"),
            Fields::Unit => panic!("unit struct not supported"),
        },
        Data::Enum(data_enum) => quote_enum_and_impl(&struct_name, &data_enum),
        _ => panic!("the DamlData attribute may only be applied to struct and enum types"),
    };
    let expanded = quote!(
        #tokens
    );
    proc_macro::TokenStream::from(expanded)
}
