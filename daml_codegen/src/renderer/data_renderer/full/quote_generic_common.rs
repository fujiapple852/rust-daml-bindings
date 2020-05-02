use daml_lf::element::{DamlKind, DamlTypeVar};
use proc_macro2::TokenStream;

use crate::renderer::{normalize_generic_param, quote_ident};
use quote::quote;

/// Quote `<A, B, C>`
pub fn quote_unbounded_params(params: &[DamlTypeVar<'_>]) -> TokenStream {
    quote_non_empty(params, |params| {
        let all_params_tokens: Vec<_> = params.iter().map(|type_var| quote_var(type_var.var)).collect();
        quote!( < #( #all_params_tokens ),* > )
    })
}

/// Quote `<A, B: Nat, C>`
pub fn quote_bounded_params(params: &[DamlTypeVar<'_>]) -> TokenStream {
    quote_non_empty(params, |params| {
        let all_bounds_tokens: Vec<_> = params.iter().map(quote_type_var).collect();
        quote!( < #( #all_bounds_tokens ),* > )
    })
}

/// Quote `where A: DamlSerializeInto<DamlValue>, B: DamlSerializeInto<DamlValue> + Nat`
pub fn quote_serialize_where(params: &[DamlTypeVar<'_>]) -> TokenStream {
    quote_where_clause(params, quote!(DamlSerializeInto<DamlValue>))
}

/// Quote `where A: DamlDeserializeFrom, B: DamlDeserializeFrom + Nat`
pub fn quote_deserialize_where(params: &[DamlTypeVar<'_>]) -> TokenStream {
    quote_where_clause(params, quote!(DamlDeserializeFrom))
}

fn quote_where_clause(params: &[DamlTypeVar<'_>], bound_tokens: TokenStream) -> TokenStream {
    quote_non_empty(params, |params| {
        let all_bounds_tokens: Vec<_> =
            params.iter().map(|type_var| quote_type_var_with_bound(type_var, &bound_tokens)).collect();
        quote!( where #( #all_bounds_tokens ),* )
    })
}

fn quote_type_var(type_var: &DamlTypeVar<'_>) -> TokenStream {
    let var_tokens = quote_var(type_var.var);
    if let DamlKind::Nat = type_var.kind {
        quote!(#var_tokens: Nat)
    } else {
        quote!(#var_tokens)
    }
}

fn quote_type_var_with_bound(type_var: &DamlTypeVar<'_>, bound: &TokenStream) -> TokenStream {
    let var_tokens = quote_var(type_var.var);
    if let DamlKind::Nat = type_var.kind {
        quote!(#var_tokens: #bound + Nat)
    } else {
        quote!(#var_tokens: #bound)
    }
}

fn quote_var(var: &str) -> TokenStream {
    quote_ident(normalize_generic_param(var).to_uppercase())
}

fn quote_non_empty<F>(params: &[DamlTypeVar<'_>], f: F) -> TokenStream
where
    F: Fn(&[DamlTypeVar<'_>]) -> TokenStream,
{
    if params.is_empty() {
        quote!()
    } else {
        f(params)
    }
}
