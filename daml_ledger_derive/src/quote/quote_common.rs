use crate::daml_element::DamlField;
use crate::daml_element::DamlType;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::convert::AsRef;
use syn::Path;

/// Quote a string as an identifier.
pub fn quote_ident(value: impl AsRef<str>) -> TokenStream {
    let ident = Ident::new(value.as_ref(), Span::call_site());
    quote!(#ident)
}
/// Quote a string as a path, i.e. `foo::bar::MyType`.
pub fn quote_path(value: impl AsRef<str>) -> TokenStream {
    let path: Path = syn::parse_str(value.as_ref()).expect("failed to parse path");
    quote!(#path)
}

/// Recursively quote a DAML type.
pub fn quote_type(daml_type: &DamlType) -> TokenStream {
    match daml_type {
        DamlType::List(inner) | DamlType::TextMap(inner) | DamlType::Optional(inner) => {
            let inner = quote_type(inner);
            let outer = quote_ident(daml_type.name());
            quote!(#outer<#inner>)
        },
        DamlType::Data(inner) => {
            let ident = quote_path(inner);
            quote!(#ident)
        },
        _ => {
            let ident = quote_ident(daml_type.name());
            quote!(#ident)
        },
    }
}

/// Recursively quote a DAML `new_xxx()` expression.
pub fn quote_new_value_expression(daml_type: &DamlType) -> TokenStream {
    match daml_type {
        DamlType::List(inner) => {
            let inner = quote_new_value_expression(inner);
            quote!(DamlValue::new_list(value.into_iter().map(|value| #inner ).collect::<Vec<_>>()))
        },
        DamlType::TextMap(inner) => {
            let inner = quote_new_value_expression(inner);
            quote!(
                DamlValue::new_map(value
                    .into_iter()
                    .map(|(k, value)| (k, #inner ))
                    .collect::<HashMap<_, _>>())
            )
        },
        DamlType::Optional(inner) => {
            let inner = quote_new_value_expression(inner);
            quote!(DamlValue::new_optional(value.map(|value| #inner )))
        },
        DamlType::Data(_) => quote!(value.into()),
        _ => {
            let (new_method_name, with_param) = daml_type.new_method();
            let new_method_name_tokens = quote_ident(new_method_name);
            if with_param {
                quote!(DamlValue::#new_method_name_tokens(value))
            } else {
                quote!(DamlValue::#new_method_name_tokens())
            }
        },
    }
}

/// Recursively quote a DAML `try_xxx()` expression.
pub fn quote_try_expression(daml_type: &DamlType) -> TokenStream {
    match daml_type {
        DamlType::List(inner) => {
            let inner = quote_try_expression(inner);
            quote!(value
                   .try_take_list()?
                   .into_iter()
                   .map(|value| #inner )
                   .collect::<DamlResult<DamlList<_>>>()
            )
        },
        DamlType::TextMap(inner) => {
            let inner = quote_try_expression(inner);
            quote!(
                value.try_take_map()?
                    .into_iter()
                    .map(|(k, value)| Ok((k, #inner? )) )
                    .collect::<DamlResult<DamlTextMap<_>>>()
            )
        },
        DamlType::Optional(inner) => {
            let inner = quote_try_expression(inner);
            quote!(value.try_take_optional()?.map(|value| #inner ).transpose())
        },
        DamlType::Data(_) => {
            let type_name = quote_path(daml_type.name());
            quote!({let data: DamlResult<#type_name> = value.try_into(); data})
        },
        _ => {
            let try_method_name = quote_ident(daml_type.try_method());
            let type_name = quote_ident(daml_type.name());
            quote!({let field: DamlResult<#type_name> = value.#try_method_name().map(Into::into); field})
        },
    }
}

/// Quote the arguments to a method.
pub fn quote_method_arguments(args: &[DamlField]) -> TokenStream {
    let all: Vec<_> = args
        .iter()
        .map(
            |DamlField {
                 field_label,
                 field_type,
             }| {
                let field_label = quote_ident(field_label);
                let field_type_rendered = quote_type(field_type);
                quote!(#field_label: impl Into<#field_type_rendered>)
            },
        )
        .collect();
    quote!( #( #all ,)* )
}
