use crate::daml_element::{extract_variant_data, DamlVariant};
use crate::quote::{quote_ident, quote_new_value_expression, quote_try_expression, quote_type};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DataEnum;

/// Generate the variant `enum` and the `From` and `TryFrom` impls.
pub fn quote_enum_and_impl(enum_name: &str, data_enum: &DataEnum) -> TokenStream {
    let variants = extract_variant_data(&data_enum);
    let enum_tokens = quote_enum(&enum_name, &variants);
    let from_trait_impl_tokens = quote_from_trait_impl(&enum_name, &variants);
    let try_from_trait_impl_tokens = quote_try_from_trait_impl(&enum_name, &variants);
    quote!(
        #enum_tokens
        #from_trait_impl_tokens
        #try_from_trait_impl_tokens
    )
}

/// Generate `enum Foo {...}` enum.
fn quote_enum(enum_name: &str, variants: &[DamlVariant]) -> TokenStream {
    let enum_name_tokens = quote_ident(enum_name);
    let body_tokens = quote_enum_body(&variants);
    quote!(
        #[derive(Eq, PartialEq, Clone, Debug)]
        pub enum #enum_name_tokens {
            #body_tokens
        }
    )
}

/// Generate the variant enum body.
fn quote_enum_body(variants: &[DamlVariant]) -> TokenStream {
    let all: Vec<_> = variants
        .iter()
        .map(
            |DamlVariant {
                 variant_name,
                 variant_type,
             }| {
                let variant_name = quote_ident(variant_name);
                match variant_type {
                    None => quote!(#variant_name),
                    Some(data) => {
                        let data = quote_type(data);
                        quote!(#variant_name(#data))
                    },
                }
            },
        )
        .collect();
    quote!( #( #all ,)* )
}

/// Generate the `From<Foo> for DamlValue` method.
fn quote_from_trait_impl(enum_name: &str, variants: &[DamlVariant]) -> TokenStream {
    let enum_name_tokens = quote_ident(enum_name);
    let all_match_arms: Vec<_> =
        variants.iter().map(|variant| quote_from_trait_match_arm(enum_name, variant)).collect();
    quote! {
        impl From<#enum_name_tokens> for DamlValue {
            fn from(value: #enum_name_tokens) -> Self {
                match value {
                    #( #all_match_arms ),*
                }
            }
        }
    }
}

/// Generate the `TryFrom<DamlValue> for Foo` method.
fn quote_try_from_trait_impl(enum_name: &str, args: &[DamlVariant]) -> TokenStream {
    let enum_name_tokens = quote_ident(enum_name);
    let all_match_arms: Vec<_> =
        args.iter().map(|variant| quote_try_from_trait_match_arm(enum_name, variant)).collect();
    let all_variant_types_string = args
        .iter()
        .map(
            |DamlVariant {
                 variant_name,
                 ..
             }| variant_name,
        )
        .join(", ");
    quote!(
        impl TryFrom<DamlValue> for #enum_name_tokens {
            type Error = DamlError;
            fn try_from(value: DamlValue) -> Result<Self, Self::Error> {
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
/// `EnumName::Variant(value) => {...}`
fn quote_from_trait_match_arm(enum_name: &str, variant: &DamlVariant) -> TokenStream {
    let enum_name_tokens = quote_ident(enum_name);
    let name = quote_ident(&variant.variant_name);
    let variant_string = &variant.variant_name;
    match &variant.variant_type {
        Some(vt) => {
            let rendered_new_value_tokens = quote_new_value_expression(vt);
            quote!(
                #enum_name_tokens::#name(value) => DamlValue::new_variant(DamlVariant::new(#variant_string, Box::new(#rendered_new_value_tokens), None))
            )
        },
        None => quote!(
            #enum_name_tokens::#name => DamlValue::new_variant(DamlVariant::new(#variant_string, Box::new(DamlValue::new_unit()), None))
        ),
    }
}

/// Quote a match arm of the `TryFrom<DamlValue> for Foo` `impl` block.
///
/// `"Variant" => Ok(EnumName::Variant(...))`
fn quote_try_from_trait_match_arm(enum_name: &str, variant: &DamlVariant) -> TokenStream {
    let enum_name_tokens = quote_ident(enum_name);
    let variant_name_tokens = quote_ident(&variant.variant_name);
    let variant_string_tokens = &variant.variant_name;
    match &variant.variant_type {
        Some(vt) => {
            let try_field_expression_tokens = quote_try_expression(vt);
            quote!(
                #variant_string_tokens => Ok(#enum_name_tokens::#variant_name_tokens({
                    let value = *variant.take_value();
                    #try_field_expression_tokens?
                }))
            )
        },
        None => quote!(
            #variant_string_tokens => Ok(#enum_name_tokens::#variant_name_tokens)
        ),
    }
}
