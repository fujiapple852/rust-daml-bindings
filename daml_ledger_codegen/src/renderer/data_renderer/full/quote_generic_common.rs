use crate::element::DamlTypeVar;
use proc_macro2::TokenStream;

use crate::renderer::{normalize_generic_param, quote_ident};
use quote::quote;

pub fn quote_generic_param_list(params: &[DamlTypeVar<'_>]) -> TokenStream {
    if params.is_empty() {
        quote!()
    } else {
        let all_params_tokens = map_generic_params(params, |p| quote!(#p));
        quote!( < #all_params_tokens > )
    }
}

pub fn quote_serialize_generic_trait_bounds(params: &[DamlTypeVar<'_>]) -> TokenStream {
    if params.is_empty() {
        quote!()
    } else {
        let all_bounds_tokens = map_generic_params(params, |p| quote!(#p: DamlSerializeInto<DamlValue>));
        quote!( where #all_bounds_tokens )
    }
}

pub fn quote_deserialize_generic_trait_bounds(params: &[DamlTypeVar<'_>]) -> TokenStream {
    if params.is_empty() {
        quote!()
    } else {
        let all_bounds_tokens = map_generic_params(params, |p| quote!(#p: DamlDeserializeFrom));
        quote!( where #all_bounds_tokens )
    }
}

fn map_generic_params(params: &[DamlTypeVar<'_>], f: impl Fn(&TokenStream) -> TokenStream) -> TokenStream {
    let all_params: Vec<_> =
        params.iter().map(|param| quote_ident(normalize_generic_param(param.var).to_uppercase())).collect();
    let all_bounds: Vec<_> = all_params.iter().map(|p| f(p)).collect();
    quote!( #( #all_bounds ),* )
}
