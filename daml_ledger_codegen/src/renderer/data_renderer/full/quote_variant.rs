use itertools::Itertools;
use proc_macro2::TokenStream;

use quote::quote;

use crate::element::{DamlField, DamlType, DamlVariant};
use crate::renderer::expression_renderer::{quote_new_value_expression, quote_try_expression, VALUE_IDENT};
use crate::renderer::type_renderer::quote_type;
use crate::renderer::{is_supported_type, quote_escaped_ident};

/// Generate the variant `enum` and the `From` and `TryFrom` impls.
pub fn quote_daml_variant(variant: &DamlVariant) -> TokenStream {
    let supported_fields: Vec<_> = variant.fields.iter().filter(|&field| is_supported_type(&field.ty)).collect();

    let variant_tokens = quote_variant(&variant.name, &supported_fields);
    let from_trait_impl_tokens = quote_from_trait_impl(&variant.name, &supported_fields);
    let try_from_trait_impl_tokens = quote_try_from_trait_impl(&variant.name, &supported_fields);
    quote!(
        #variant_tokens
        #from_trait_impl_tokens
        #try_from_trait_impl_tokens
    )
}

/// Generate `enum Foo {...}` variant.
fn quote_variant(variant_name: &str, variants: &[&DamlField]) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(variant_name);
    let body_tokens = quote_variant_body(&variants);
    quote!(
        #[derive(Eq, PartialEq, Clone, Debug)]
        pub enum #enum_name_tokens {
            #body_tokens
        }
    )
}

/// Generate the variant body.
fn quote_variant_body(variants: &[&DamlField]) -> TokenStream {
    let all: Vec<_> = variants
        .iter()
        .map(
            |DamlField {
                 name,
                 ty,
             }| {
                let variant_name = quote_escaped_ident(name);

                if let DamlType::Unit = ty {
                    quote!(#variant_name)
                } else {
                    let data = quote_type(ty);
                    quote!(#variant_name(#data))
                }
            },
        )
        .collect();
    quote!( #( #all ,)* )
}

/// Generate the `From<Foo> for DamlValue` method.
fn quote_from_trait_impl(variant_name: &str, variants: &[&DamlField]) -> TokenStream {
    let variant_name_tokens = quote_escaped_ident(variant_name);
    let all_match_arms: Vec<_> =
        variants.iter().map(|variant| quote_from_trait_match_arm(variant_name, variant)).collect();
    quote! {
        impl From<#variant_name_tokens> for DamlValue {
            fn from(value: #variant_name_tokens) -> Self {
                match value {
                    #( #all_match_arms ),*
                }
            }
        }
    }
}

/// Generate the `TryFrom<DamlValue> for Foo` method.
fn quote_try_from_trait_impl(variant_name: &str, args: &[&DamlField]) -> TokenStream {
    let variant_name_tokens = quote_escaped_ident(variant_name);
    let all_match_arms: Vec<_> =
        args.iter().map(|variant| quote_try_from_trait_match_arm(variant_name, variant)).collect();
    let all_variant_types_string = args
        .iter()
        .map(
            |DamlField {
                 name,
                 ..
             }| name,
        )
        .join(", ");
    quote!(
        impl TryFrom<DamlValue> for #variant_name_tokens {
            type Error = DamlError;
            fn try_from(value: DamlValue) -> std::result::Result<Self, <#variant_name_tokens as TryFrom<DamlValue>>::Error> {
                let variant = value.try_take_variant()?;
                match variant.constructor() {
                    #( #all_match_arms, )*
                    ctor => Err(DamlError::UnexpectedVariant(#all_variant_types_string.to_owned(), ctor.to_owned())),
                }
            }
        }
    )
}

/// Quote a match arm of the `From<Foo> for DamlValue` `impl` block.
///
/// `VariantName::Variant(value) => {...}`
fn quote_from_trait_match_arm(variant_name: &str, variant: &DamlField) -> TokenStream {
    let variant_name_tokens = quote_escaped_ident(variant_name);
    let name = quote_escaped_ident(variant.name);
    let variant_string = variant.name;

    if let DamlType::Unit = variant.ty {
        quote!(
            #variant_name_tokens::#name => DamlValue::new_variant(DamlVariant::new(#variant_string, Box::new(DamlValue::new_unit()), None))
        )
    } else {
        let rendered_new_value_tokens = quote_new_value_expression(VALUE_IDENT, &variant.ty);
        quote!(
            #variant_name_tokens::#name(value) => DamlValue::new_variant(DamlVariant::new(#variant_string, Box::new(#rendered_new_value_tokens), None))
        )
    }
}

/// Quote a match arm of the `TryFrom<DamlValue> for Foo` `impl` block.
///
/// `"Variant" => Ok(VariantName::Variant(...))`
fn quote_try_from_trait_match_arm(variant_name: &str, variant: &DamlField) -> TokenStream {
    let variant_name_tokens = quote_escaped_ident(variant_name);
    let variant_constructor_name_tokens = quote_escaped_ident(variant.name);
    let variant_constructor_string_tokens = variant.name;

    if let DamlType::Unit = &variant.ty {
        quote!(
            #variant_constructor_string_tokens => Ok(#variant_name_tokens::#variant_constructor_name_tokens)
        )
    } else {
        let try_field_expression_tokens = quote_try_expression(VALUE_IDENT, &variant.ty);
        quote!(
            #variant_constructor_string_tokens => Ok(#variant_name_tokens::#variant_constructor_name_tokens({
                let value = *variant.take_value();
                #try_field_expression_tokens?
            }))
        )
    }
}
