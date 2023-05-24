use daml_lf::element::{DamlKind, DamlTypeVarWithKind};
use proc_macro2::TokenStream;

use crate::renderer::{normalize_generic_param, quote_ident};
use quote::quote;

/// Quote `<A, B, C>`
pub fn quote_unbounded_params(params: &[DamlTypeVarWithKind<'_>]) -> TokenStream {
    quote_non_empty(params, |params| {
        let all_params_tokens: Vec<_> = params.iter().map(|type_var| quote_var(type_var.var())).collect();
        quote!( < #( #all_params_tokens ),* > )
    })
}

/// Quote `<A, B: Nat, C>`
pub fn quote_bounded_params(params: &[DamlTypeVarWithKind<'_>]) -> TokenStream {
    quote_non_empty(params, |params| {
        let all_bounds_tokens: Vec<_> = params.iter().map(quote_type_var).collect();
        quote!( < #( #all_bounds_tokens ),* > )
    })
}

/// Quote `where A: DamlSerializeInto<DamlValue>, B: DamlSerializeInto<DamlValue> + Nat`
pub fn quote_serialize_where(params: &[DamlTypeVarWithKind<'_>]) -> TokenStream {
    quote_where_clause(params, quote!(DamlSerializeInto<DamlValue>))
}

/// Quote `where A: DamlDeserializeFrom + Ord, B: DamlDeserializeFrom + Ord + Nat`
pub fn quote_deserialize_where(params: &[DamlTypeVarWithKind<'_>]) -> TokenStream {
    quote_where_clause(params, quote!(DamlDeserializeFrom + Ord))
}

fn quote_where_clause(params: &[DamlTypeVarWithKind<'_>], bound_tokens: TokenStream) -> TokenStream {
    quote_non_empty(params, |params| {
        let all_bounds_tokens: Vec<_> =
            params.iter().map(|type_var| quote_type_var_with_bound(type_var, &bound_tokens)).collect();
        quote!( where #( #all_bounds_tokens ),* )
    })
}

fn quote_type_var(type_var: &DamlTypeVarWithKind<'_>) -> TokenStream {
    let var_tokens = quote_var(type_var.var());
    if matches!(type_var.kind(), DamlKind::Nat) {
        quote!(#var_tokens: Nat)
    } else {
        quote!(#var_tokens)
    }
}

fn quote_type_var_with_bound(type_var: &DamlTypeVarWithKind<'_>, bound: &TokenStream) -> TokenStream {
    let var_tokens = quote_var(type_var.var());
    if matches!(type_var.kind(), DamlKind::Nat) {
        quote!(#var_tokens: #bound + Nat)
    } else {
        quote!(#var_tokens: #bound)
    }
}

fn quote_var(var: &str) -> TokenStream {
    quote_ident(normalize_generic_param(var).to_uppercase())
}

fn quote_non_empty<F>(params: &[DamlTypeVarWithKind<'_>], f: F) -> TokenStream
where
    F: Fn(&[DamlTypeVarWithKind<'_>]) -> TokenStream,
{
    if params.is_empty() {
        quote!()
    } else {
        f(params)
    }
}
