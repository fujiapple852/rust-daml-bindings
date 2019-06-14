use crate::daml_element::DamlTemplateInfo;
use crate::quote::{
    quote_contract_struct_and_impl, quote_make_create_command_method, quote_package_id_method, quote_struct_and_impl,
};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields};

pub fn generate_template(input: DeriveInput, template_info: &DamlTemplateInfo) -> proc_macro::TokenStream {
    let struct_name = input.ident.to_string();
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields_named),
            ..
        }) => {
            let struct_and_impl_tokens = quote_struct_and_impl(&struct_name, fields_named);
            let package_id_method_tokens = quote_package_id_method(&struct_name, template_info);
            let make_create_method_tokens = quote_make_create_command_method(&struct_name);
            let contract_struct_and_impl_tokens = quote_contract_struct_and_impl(&struct_name);
            let expanded = quote!(
                #struct_and_impl_tokens
                #package_id_method_tokens
                #make_create_method_tokens
                #contract_struct_and_impl_tokens
            );
            proc_macro::TokenStream::from(expanded)
        },
        _ => panic!("the DamlTemplate attribute may only be applied to a named struct type"),
    }
}
