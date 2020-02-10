use crate::element::{DamlField, DamlType};
use crate::renderer::type_renderer::{quote_data_ref, quote_type};
use crate::renderer::{quote_escaped_ident, quote_ident};
use proc_macro2::TokenStream;
use quote::quote;

pub const VALUE_IDENT: &str = "value";

/// Recursively quote a DAML `new_xxx()` expression.
pub fn quote_new_value_expression(value_ident: &str, daml_type: &DamlType) -> TokenStream {
    let value_ident_tokens = quote_ident(value_ident);
    match daml_type {
        DamlType::List(inner) => {
            let inner = quote_new_value_expression(value_ident, inner);
            quote!(DamlValue::new_list(#value_ident_tokens.into_iter().map(|#value_ident_tokens| #inner ).collect::<Vec<_>>()))
        },
        DamlType::TextMap(inner) => {
            let inner = quote_new_value_expression(value_ident, inner);
            quote!(
                DamlValue::new_map(#value_ident_tokens
                    .into_iter()
                    .map(|(k, #value_ident_tokens)| (k, #inner ))
                    .collect::<HashMap<_, _>>())
            )
        },
        DamlType::Optional(inner) => {
            let inner = quote_new_value_expression(value_ident, inner);
            quote!(DamlValue::new_optional(#value_ident_tokens.map(|#value_ident_tokens| #inner )))
        },
        DamlType::DataRef(_) => quote!(#value_ident_tokens.into()),
        DamlType::BoxedDataRef(_) => quote!((*#value_ident_tokens).into()),
        DamlType::ContractId(_)
        | DamlType::Int64
        | DamlType::Numeric
        | DamlType::Text
        | DamlType::Timestamp
        | DamlType::Party
        | DamlType::Bool
        | DamlType::Unit
        | DamlType::Date
        | DamlType::Update
        | DamlType::Scenario
        | DamlType::Var
        | DamlType::Arrow
        | DamlType::Any
        | DamlType::TypeRep => {
            let (new_method_name, with_param) = new_method(daml_type);
            let new_method_name_tokens = quote_escaped_ident(new_method_name);
            if with_param {
                quote!(DamlValue::#new_method_name_tokens(#value_ident_tokens))
            } else {
                quote!(DamlValue::#new_method_name_tokens())
            }
        },
    }
}

/// Recursively quote a DAML `try_xxx()` expression.
pub fn quote_try_expression(value_ident: &str, daml_type: &DamlType) -> TokenStream {
    let value_ident_tokens = quote_ident(value_ident);
    match daml_type {
        DamlType::List(inner) => {
            let inner = quote_try_expression(value_ident, inner);
            quote!(#value_ident_tokens
                     .try_take_list()?
                     .into_iter()
                     .map(|#value_ident_tokens| #inner )
                     .collect::<DamlResult<DamlList<_>>>()
            )
        },
        DamlType::TextMap(inner) => {
            let inner = quote_try_expression(value_ident, inner);
            quote!(#value_ident_tokens
                    .try_take_map()?
                    .into_iter()
                    .map(|(k, #value_ident_tokens)| Ok((k, #inner? )) )
                    .collect::<DamlResult<DamlTextMap<_>>>()
            )
        },
        DamlType::Optional(inner) => {
            let inner = quote_try_expression(value_ident, inner);
            quote!(#value_ident_tokens.try_take_optional()?.map(|#value_ident_tokens| #inner ).transpose())
        },
        DamlType::DataRef(data_ref) => {
            let type_tokens = quote_data_ref(data_ref);
            quote!({let data: DamlResult<#type_tokens> = #value_ident_tokens.try_into(); data})
        },
        DamlType::BoxedDataRef(_) => quote!(#value_ident_tokens.try_into().map(Box::new)),
        DamlType::ContractId(_)
        | DamlType::Int64
        | DamlType::Numeric
        | DamlType::Text
        | DamlType::Timestamp
        | DamlType::Party
        | DamlType::Bool
        | DamlType::Unit
        | DamlType::Date
        | DamlType::Update
        | DamlType::Scenario
        | DamlType::Var
        | DamlType::Arrow
        | DamlType::Any
        | DamlType::TypeRep => {
            let try_method_name = quote_ident(try_method(daml_type));
            let type_name = quote_ident(daml_type.name());
            quote!({let field: DamlResult<#type_name> = #value_ident_tokens.#try_method_name().map(Into::into); field})
        },
    }
}

/// Quote the arguments to a method.
pub fn quote_method_arguments(args: &[&DamlField]) -> TokenStream {
    let all: Vec<_> = args
        .iter()
        .map(
            |DamlField {
                 name,
                 ty,
             }| {
                let field_label = quote_escaped_ident(name);
                let field_type_rendered = quote_type(ty);
                quote!(#field_label: impl Into<#field_type_rendered>)
            },
        )
        .collect();
    quote!( #( #all ,)* )
}

fn new_method(daml_type: &DamlType) -> (String, bool) {
    let with_param = if let DamlType::Unit = daml_type {
        false
    } else {
        true
    };
    let new_method_name = format!("new_{}", get_type_method(daml_type));
    (new_method_name, with_param)
}

fn try_method(daml_type: &DamlType) -> String {
    match get_type_method(daml_type) {
        "numeric" => "try_numeric_clone".to_owned(),
        type_method => format!("try_{}", type_method),
    }
}

fn get_type_method<'a>(daml_type: &'a DamlType<'a>) -> &'a str {
    match daml_type {
        DamlType::ContractId(_) => "contract_id",
        DamlType::Int64 => "int64",
        DamlType::Numeric => "numeric",
        DamlType::Text => "text",
        DamlType::Timestamp => "timestamp",
        DamlType::Party => "party",
        DamlType::Bool => "bool",
        DamlType::Unit => "unit",
        DamlType::Date => "date",
        _ => panic!("internal error, get_type_method called for non-primitive type"),
    }
}
