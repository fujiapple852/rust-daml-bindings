use crate::element::DamlField;
use crate::renderer::quote_escaped_ident;
use crate::renderer::type_renderer::quote_type;
use proc_macro2::TokenStream;
use quote::quote;

/// Quote the arguments to a method.
pub fn quote_method_arguments(fields: &[&DamlField<'_>]) -> TokenStream {
    let all_fields: Vec<_> = fields
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
    quote!( #( #all_fields ,)* )
}
