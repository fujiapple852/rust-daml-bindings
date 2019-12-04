use std::iter;

use heck::SnakeCase;
use proc_macro2::TokenStream;

use quote::quote;

use crate::element::*;
use crate::renderer::quote_ident;
use crate::renderer::renderer_utils::quote_escaped_ident;

#[allow(clippy::match_same_arms)]
pub fn quote_type(daml_type: &DamlType) -> TokenStream {
    match daml_type {
        DamlType::List(nested) | DamlType::TextMap(nested) | DamlType::Optional(nested) => {
            let prim_name_tokens = quote_escaped_ident(daml_type.name());
            let prim_param_tokens = quote_type(nested);
            quote!(
                #prim_name_tokens<#prim_param_tokens>
            )
        },
        // Ignoring ContractId inner type
        DamlType::ContractId(_) => quote_escaped_ident(daml_type.name()),
        DamlType::DataRef(data_ref) => quote_data_ref(data_ref),
        DamlType::BoxedDataRef(data_ref) => {
            let data_ref = quote_data_ref(data_ref);
            quote!(Box<#data_ref>)
        },
        _ => quote_escaped_ident(daml_type.name()),
    }
}

pub fn quote_data_ref(data_ref: &DamlDataRef) -> TokenStream {
    match data_ref {
        DamlDataRef::Local(local_data_ref) => quote_escaped_ident(&local_data_ref.data_name),
        DamlDataRef::NonLocal(non_local_data_ref) => {
            let target_type_tokens = quote_escaped_ident(&non_local_data_ref.data_name);
            let target_path_tokens = quote_non_local_path(non_local_data_ref);
            quote!(#target_path_tokens #target_type_tokens)
        },
        DamlDataRef::Absolute(abs_data_ref) => {
            let target_type_tokens = quote_escaped_ident(&abs_data_ref.data_name);
            let target_path_tokens = quote_absolute_data_ref(abs_data_ref);
            quote!(#target_path_tokens #target_type_tokens)
        },
    }
}

fn quote_absolute_data_ref(abs_data_ref: &DamlAbsoluteDataRef) -> TokenStream {
    let path: Vec<&str> = if abs_data_ref.package_name.is_empty() {
        abs_data_ref.module_path.iter().map(AsRef::as_ref).collect()
    } else {
        iter::once(abs_data_ref.package_name).chain(abs_data_ref.module_path.iter().map(AsRef::as_ref)).collect()
    };
    let target_path_tokens: Vec<_> = path.into_iter().map(SnakeCase::to_snake_case).map(quote_escaped_ident).collect();
    quote!(
        crate :: #( #target_path_tokens :: )*
    )
}

fn quote_non_local_path(data_ref: &DamlNonLocalDataRef) -> TokenStream {
    let current_full_path: Vec<_> = iter::once(data_ref.source_package_name)
        .chain(data_ref.source_module_path.iter().map(AsRef::as_ref))
        .map(SnakeCase::to_snake_case)
        .collect();
    let target_full_path: Vec<_> = iter::once(data_ref.target_package_name)
        .chain(data_ref.target_module_path.iter().map(AsRef::as_ref))
        .map(SnakeCase::to_snake_case)
        .collect();
    let common_prefix_length =
        current_full_path.iter().zip(target_full_path.iter()).take_while(|(a, b)| a == b).count();
    let super_prefix_tokens: Vec<_> =
        iter::repeat(quote_ident("super")).take(current_full_path.len() - common_prefix_length).collect();
    let relative_path_tokens: Vec<_> = target_full_path
        .iter()
        .skip(common_prefix_length)
        .map(String::as_str)
        .map(SnakeCase::to_snake_case)
        .map(quote_escaped_ident)
        .collect();
    quote!(
        #( #super_prefix_tokens :: )* #( #relative_path_tokens :: )*
    )
}
