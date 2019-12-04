use itertools::Itertools;
use proc_macro2::TokenStream;

use quote::quote;

use crate::element::DamlEnum;
use crate::renderer::quote_escaped_ident;

/// Generate the `enum` and the `From` and `TryFrom` impls.
pub fn quote_daml_enum(daml_enum: &DamlEnum) -> TokenStream {
    let enum_tokens = quote_enum(&daml_enum.name, daml_enum.constructors.as_slice());
    let from_trait_impl_tokens = quote_from_trait_impl(&daml_enum.name, daml_enum.constructors.as_slice());
    let try_from_trait_impl_tokens = quote_try_from_trait_impl(&daml_enum.name, daml_enum.constructors.as_slice());
    quote!(
        #enum_tokens
        #from_trait_impl_tokens
        #try_from_trait_impl_tokens
    )
}

/// Generate `enum Foo {...}` enum.
fn quote_enum(enum_name: &str, variants: &[&str]) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(enum_name);
    let body_tokens = quote_enum_body(&variants);
    quote!(
        #[derive(Eq, PartialEq, Clone, Debug)]
        pub enum #enum_name_tokens {
            #body_tokens
        }
    )
}

/// Generate the enum body.
fn quote_enum_body(enum_variants: &[&str]) -> TokenStream {
    let all: Vec<_> = enum_variants
        .iter()
        .map(|s| {
            let variant_name = quote_escaped_ident(s);
            quote!(#variant_name)
        })
        .collect();
    quote!( #( #all ,)* )
}

/// Generate the `From<Foo> for DamlValue` method.
fn quote_from_trait_impl(enum_name: &str, enum_variants: &[&str]) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(enum_name);
    let all_match_arms: Vec<_> =
        enum_variants.iter().map(|enum_variant| quote_from_trait_match_arm(enum_name, enum_variant)).collect();
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
fn quote_try_from_trait_impl(enum_name: &str, args: &[&str]) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(enum_name);
    let all_match_arms: Vec<_> =
        args.iter().map(|enum_variant| quote_try_from_trait_match_arm(enum_name, enum_variant)).collect();
    let all_enum_variant_types_string = args.iter().join(", ");
    quote!(
        impl TryFrom<DamlValue> for #enum_name_tokens {
            type Error = DamlError;
            fn try_from(value: DamlValue) -> std::result::Result<Self, <#enum_name_tokens as TryFrom<DamlValue>>::Error> {
                let enum_variant = value.try_take_enum()?;
                match enum_variant.constructor() {
                    #( #all_match_arms, )*
                    ctor => Err(DamlError::UnexpectedVariant(#all_enum_variant_types_string.to_owned(), ctor.to_owned())),
                }
            }
        }
    )
}

/// Quote a match arm of the `From<Foo> for DamlValue` `impl` block.
///
/// `EnumName::Variant(value) => {...}`
fn quote_from_trait_match_arm(enum_name: &str, variant_name: &str) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(enum_name);
    let enum_constructor_name_tokens = quote_escaped_ident(variant_name);
    let enum_constructor_name_string = variant_name;
    quote!(
        #enum_name_tokens::#enum_constructor_name_tokens => DamlValue::new_enum(DamlEnum::new(#enum_constructor_name_string, None))
    )
}

/// Quote a match arm of the `TryFrom<DamlValue> for Foo` `impl` block.
///
/// `"EnumVariant" => Ok(EnumName::Variant(...))`
fn quote_try_from_trait_match_arm(enum_name: &str, variant_name: &str) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(enum_name);
    let enum_constructor_name_tokens = quote_escaped_ident(variant_name);
    let enum_constructor_string_tokens = variant_name;
    quote!(
        #enum_constructor_string_tokens => Ok(#enum_name_tokens::#enum_constructor_name_tokens)
    )
}
