use crate::renderer::data_renderer::full::{
    quote_bounded_params, quote_deserialize_where, quote_serialize_where, quote_unbounded_params,
};
use crate::renderer::{quote_escaped_ident, RenderContext};
use daml_lf::element::DamlEnum;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

/// Generate the `enum` and the `From` and `TryFrom` impls.
pub fn quote_daml_enum(_ctx: &RenderContext<'_>, daml_enum: &DamlEnum<'_>) -> TokenStream {
    let enum_tokens = quote_enum(daml_enum);
    let serialize_impl_tokens = quote_serialize_trait_impl(daml_enum);
    let deserialize_trait_impl_tokens = quote_deserialize_trait_impl(daml_enum);
    quote!(
        #enum_tokens
        #serialize_impl_tokens
        #deserialize_trait_impl_tokens
    )
}

/// Generate `enum Foo {...}` enum.
fn quote_enum(daml_enum: &DamlEnum<'_>) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(daml_enum.name());
    let bounded_param_tokens = quote_bounded_params(daml_enum.type_arguments());
    let unbounded_param_tokens = quote_unbounded_params(daml_enum.type_arguments());

    let body_tokens = quote_enum_body(daml_enum);
    quote!(
        #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Debug)]
        pub enum #enum_name_tokens #bounded_param_tokens {
            #body_tokens
        }
        impl #bounded_param_tokens DamlDeserializableType for #enum_name_tokens #unbounded_param_tokens {}
        impl #bounded_param_tokens DamlSerializableType for #enum_name_tokens #unbounded_param_tokens {}
    )
}

/// Generate the enum body.
fn quote_enum_body(daml_enum: &DamlEnum<'_>) -> TokenStream {
    let all: Vec<_> = daml_enum
        .constructors()
        .map(|s| {
            let variant_name = quote_escaped_ident(s);
            quote!(#variant_name)
        })
        .collect();
    quote!( #( #all ,)* )
}

/// Generate the `DamlSerializeFrom<Foo> for DamlValue` method.
fn quote_serialize_trait_impl(daml_enum: &DamlEnum<'_>) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(daml_enum.name());
    let unbounded_param_tokens = quote_unbounded_params(daml_enum.type_arguments());
    let serialize_where_tokens = quote_serialize_where(daml_enum.type_arguments());
    let all_match_arms: Vec<_> = daml_enum
        .constructors()
        .map(|enum_variant| quote_from_trait_match_arm(daml_enum.name(), enum_variant))
        .collect();
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
fn quote_deserialize_trait_impl(daml_enum: &DamlEnum<'_>) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(daml_enum.name());
    let unbounded_param_tokens = quote_unbounded_params(daml_enum.type_arguments());
    let deserialize_where_tokens = quote_deserialize_where(daml_enum.type_arguments());
    let all_match_arms: Vec<_> = daml_enum
        .constructors()
        .map(|enum_variant| quote_try_from_trait_match_arm(daml_enum.name(), enum_variant))
        .collect();
    let all_enum_variant_types_string = daml_enum.constructors().join(", ");
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
