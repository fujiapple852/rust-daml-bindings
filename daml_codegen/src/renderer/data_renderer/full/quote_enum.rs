use itertools::Itertools;
use proc_macro2::TokenStream;

use quote::quote;

use crate::renderer::data_renderer::full::{
    quote_bounded_params, quote_deserialize_where, quote_serialize_where, quote_unbounded_params,
};
use crate::renderer::{quote_escaped_ident, RenderContext};
use daml_lf::element::{DamlEnum, DamlTypeVar};

/// Generate the `enum` and the `From` and `TryFrom` impls.
pub fn quote_daml_enum(_ctx: &RenderContext<'_>, daml_enum: &DamlEnum<'_>) -> TokenStream {
    let enum_tokens = quote_enum(daml_enum.name, daml_enum.constructors.as_slice(), &daml_enum.type_arguments);
    let serialize_impl_tokens =
        quote_serialize_trait_impl(daml_enum.name, daml_enum.constructors.as_slice(), &daml_enum.type_arguments);
    let deserialize_trait_impl_tokens =
        quote_deserialize_trait_impl(daml_enum.name, daml_enum.constructors.as_slice(), &daml_enum.type_arguments);
    quote!(
        #enum_tokens
        #serialize_impl_tokens
        #deserialize_trait_impl_tokens
    )
}

/// Generate `enum Foo {...}` enum.
fn quote_enum(enum_name: &str, variants: &[&str], type_arguments: &[DamlTypeVar<'_>]) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(enum_name);
    let bounded_param_tokens = quote_bounded_params(type_arguments);
    let unbounded_param_tokens = quote_unbounded_params(type_arguments);

    let body_tokens = quote_enum_body(variants);
    quote!(
        #[derive(Eq, PartialEq, Clone, Debug)]
        pub enum #enum_name_tokens #bounded_param_tokens {
            #body_tokens
        }
        impl #bounded_param_tokens DamlDeserializableType for #enum_name_tokens #unbounded_param_tokens {}
        impl #bounded_param_tokens DamlSerializableType for #enum_name_tokens #unbounded_param_tokens {}
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

/// Generate the `DamlSerializeFrom<Foo> for DamlValue` method.
fn quote_serialize_trait_impl(
    enum_name: &str,
    enum_variants: &[&str],
    type_arguments: &[DamlTypeVar<'_>],
) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(enum_name);
    let unbounded_param_tokens = quote_unbounded_params(type_arguments);
    let serialize_where_tokens = quote_serialize_where(type_arguments);
    let all_match_arms: Vec<_> =
        enum_variants.iter().map(|enum_variant| quote_from_trait_match_arm(enum_name, enum_variant)).collect();
    quote! {
        impl #unbounded_param_tokens DamlSerializeFrom<#enum_name_tokens #unbounded_param_tokens> for DamlValue #serialize_where_tokens {
            fn serialize_from(value: #enum_name_tokens #unbounded_param_tokens) -> Self {
                match value {
                    #( #all_match_arms ),*
                }
            }
        }
    }
}

/// Generate the `DamlDeserializeFrom<DamlValue> for Foo` method.
fn quote_deserialize_trait_impl(
    enum_name: &str,
    enum_variants: &[&str],
    type_arguments: &[DamlTypeVar<'_>],
) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(enum_name);
    let unbounded_param_tokens = quote_unbounded_params(type_arguments);
    let deserialize_where_tokens = quote_deserialize_where(type_arguments);
    let all_match_arms: Vec<_> =
        enum_variants.iter().map(|enum_variant| quote_try_from_trait_match_arm(enum_name, enum_variant)).collect();
    let all_enum_variant_types_string = enum_variants.iter().join(", ");
    quote!(
        impl #unbounded_param_tokens DamlDeserializeFrom for #enum_name_tokens #unbounded_param_tokens #deserialize_where_tokens {
            fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
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
