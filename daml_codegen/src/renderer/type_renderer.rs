use crate::renderer::renderer_utils::quote_escaped_ident;
use crate::renderer::{normalize_generic_param, quote_ident};
use daml_lf::element::{DamlAbsoluteTyCon, DamlNonLocalTyCon, DamlTyCon, DamlTyConName, DamlType};
use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use std::iter;

#[allow(clippy::match_same_arms)]
pub fn quote_type(daml_type: &DamlType<'_>) -> TokenStream {
    match daml_type {
        DamlType::Numeric(inner) => {
            let prim_name_tokens = quote_escaped_ident(daml_type.name());
            let prim_param_tokens = quote_type(inner);
            quote!(
                #prim_name_tokens<#prim_param_tokens>
            )
        },
        DamlType::List(args) | DamlType::TextMap(args) | DamlType::Optional(args) => match args.as_slice() {
            [arg] => {
                let prim_name_tokens = quote_escaped_ident(daml_type.name());
                let prim_param_tokens = quote_type(arg);
                quote!(
                    #prim_name_tokens<#prim_param_tokens>
                )
            },
            _ => panic!("expected exactly 1 type argument for {}, found {:?}", daml_type.name(), args),
        },
        DamlType::GenMap(args) => match args.as_slice() {
            [k, v] => {
                let prim_name_tokens = quote_escaped_ident(daml_type.name());
                let prim_key_param_tokens = quote_type(k);
                let prim_value_param_tokens = quote_type(v);
                quote!(
                    #prim_name_tokens<#prim_key_param_tokens, #prim_value_param_tokens>
                )
            },
            _ => panic!("expected exactly 2 type argument for {}, found {:?}", daml_type.name(), args),
        },
        // Ignoring ContractId inner type
        DamlType::ContractId(_) => quote_escaped_ident(daml_type.name()),
        DamlType::TyCon(tycon) => quote_tycon(tycon),
        DamlType::BoxedTyCon(tycon) => {
            let tycon = quote_tycon(tycon);
            quote!(Box<#tycon>)
        },
        DamlType::Var(var) => {
            let var_tokens = quote_ident(normalize_generic_param(var.var()).to_uppercase());
            quote!(#var_tokens)
        },
        DamlType::Nat(n) => quote_ident(format!("{}{}", daml_type.name(), n)),
        DamlType::Int64
        | DamlType::Text
        | DamlType::Timestamp
        | DamlType::Party
        | DamlType::Bool
        | DamlType::Unit
        | DamlType::Date => quote_escaped_ident(daml_type.name()),
        DamlType::Update
        | DamlType::Scenario
        | DamlType::Arrow
        | DamlType::Any
        | DamlType::TypeRep
        | DamlType::Bignumeric
        | DamlType::RoundingMode
        | DamlType::Forall(_)
        | DamlType::Struct(_)
        | DamlType::Syn(_) => panic!("cannot render unsupported type: {}", daml_type.name()),
    }
}

pub fn quote_tycon(tycon: &DamlTyCon<'_>) -> TokenStream {
    let type_arguments_tokens = quote_generic_type_arguments(tycon.type_arguments());
    match tycon.tycon() {
        DamlTyConName::Local(local_tycon) => {
            let target_type_tokens = quote_escaped_ident(local_tycon.data_name());
            quote!(#target_type_tokens #type_arguments_tokens)
        },
        DamlTyConName::NonLocal(non_local_tycon) => {
            let target_type_tokens = quote_escaped_ident(non_local_tycon.data_name());
            let target_path_tokens = quote_non_local_path(non_local_tycon);
            quote!(#target_path_tokens #target_type_tokens #type_arguments_tokens)
        },
        DamlTyConName::Absolute(abs_tycon) => {
            let target_type_tokens = quote_escaped_ident(abs_tycon.data_name());
            let target_path_tokens = quote_absolute_tycon(abs_tycon);
            quote!(#target_path_tokens #target_type_tokens #type_arguments_tokens)
        },
    }
}

fn quote_generic_type_arguments(type_arguments: &[DamlType<'_>]) -> TokenStream {
    if type_arguments.is_empty() {
        quote!()
    } else {
        let all_type_arguments: Vec<_> = type_arguments.iter().map(quote_type).collect();
        quote!( < #( #all_type_arguments ),* > )
    }
}

fn quote_absolute_tycon(abs_tycon: &DamlAbsoluteTyCon<'_>) -> TokenStream {
    let path: Vec<&str> = if abs_tycon.package_name().is_empty() {
        abs_tycon.module_path().map(AsRef::as_ref).collect()
    } else {
        iter::once(abs_tycon.package_name()).chain(abs_tycon.module_path().map(AsRef::as_ref)).collect()
    };
    let target_path_tokens: Vec<_> = path.into_iter().map(SnakeCase::to_snake_case).map(quote_escaped_ident).collect();
    quote!(
        crate :: #( #target_path_tokens :: )*
    )
}

fn quote_non_local_path(tycon: &DamlNonLocalTyCon<'_>) -> TokenStream {
    let current_full_path: Vec<_> = iter::once(tycon.source_package_name())
        .chain(tycon.source_module_path().map(AsRef::as_ref))
        .map(SnakeCase::to_snake_case)
        .collect();
    let target_full_path: Vec<_> = iter::once(tycon.target_package_name())
        .chain(tycon.target_module_path().map(AsRef::as_ref))
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
