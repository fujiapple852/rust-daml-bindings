use itertools::Itertools;
use proc_macro2::TokenStream;

use quote::quote;

use crate::renderer::data_renderer::full::{
    quote_bounded_params, quote_deserialize_where, quote_serialize_where, quote_unbounded_params,
};
use crate::renderer::type_renderer::quote_type;
use crate::renderer::{normalize_generic_param, quote_escaped_ident, quote_ident, IsRenderable, RenderContext};
use daml_lf::element::{DamlField, DamlType, DamlTypeVarWithKind, DamlVariant};
use std::ops::Not;

/// Generate the variant `enum` and the `DamlDeserializeFrom` and `DamlDeserializeFrom` impls.
pub fn quote_daml_variant(ctx: &RenderContext<'_>, variant: &DamlVariant<'_>) -> TokenStream {
    let supported_fields: Vec<_> =
        variant.fields().iter().filter(|&field| IsRenderable::new(ctx).check_type(field.ty())).collect();
    let variant_tokens = quote_variant(variant.name(), &supported_fields, variant.type_params());
    let serialize_trait_impl_tokens =
        quote_serialize_trait_impl(variant.name(), &supported_fields, variant.type_params());
    let deserialize_trait_impl_tokens =
        quote_deserialize_trait_impl(variant.name(), &supported_fields, variant.type_params());
    quote!(
        #variant_tokens
        #serialize_trait_impl_tokens
        #deserialize_trait_impl_tokens
    )
}

/// Generate `enum Foo {...}` variant.
fn quote_variant(variant_name: &str, variants: &[&DamlField<'_>], params: &[DamlTypeVarWithKind<'_>]) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(variant_name);
    let bounded_param_tokens = quote_bounded_params(params);
    let unbounded_param_tokens = quote_unbounded_params(params);
    let body_tokens = quote_variant_body(variants);
    let phantom_tokens = quote_unused_phantom_params(params, variants);
    quote!(
        #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Debug)]
        pub enum #enum_name_tokens #bounded_param_tokens {
            #body_tokens
            #phantom_tokens
        }
        impl #bounded_param_tokens DamlDeserializableType for #enum_name_tokens #unbounded_param_tokens {}
        impl #bounded_param_tokens DamlSerializableType for #enum_name_tokens #unbounded_param_tokens {}
    )
}

/// Generate the variant body.
fn quote_variant_body(variants: &[&DamlField<'_>]) -> TokenStream {
    let all: Vec<_> = variants
        .iter()
        .map(|&field| {
            let variant_name = quote_escaped_ident(field.name());
            if matches!(field.ty(), DamlType::Unit) {
                quote!(#variant_name)
            } else {
                let data = quote_type(field.ty());
                quote!(#variant_name(#data))
            }
        })
        .collect();
    quote!( #( #all ,)* )
}

/// Generate the `DamlSerializeFrom<Foo> for DamlValue` method.
fn quote_serialize_trait_impl(
    variant_name: &str,
    variants: &[&DamlField<'_>],
    params: &[DamlTypeVarWithKind<'_>],
) -> TokenStream {
    let variant_name_tokens = quote_escaped_ident(variant_name);
    let unbounded_param_tokens = quote_unbounded_params(params);
    let serialize_where_tokens = quote_serialize_where(params);
    let all_match_arms: Vec<_> =
        variants.iter().map(|variant| quote_from_trait_match_arm(variant_name, variant)).collect();
    quote! {
        impl #unbounded_param_tokens DamlSerializeFrom<#variant_name_tokens #unbounded_param_tokens> for DamlValue #serialize_where_tokens {
            fn serialize_from(value: #variant_name_tokens #unbounded_param_tokens) -> Self {
                match value {
                    #( #all_match_arms ,)*
                    _ => panic!("type {} cannot be serialized", stringify!(#variant_name_tokens))
                }
            }
        }
    }
}

/// Generate the `DamlDeserializeFrom for Foo` method.
fn quote_deserialize_trait_impl(
    variant_name: &str,
    match_arms: &[&DamlField<'_>],
    params: &[DamlTypeVarWithKind<'_>],
) -> TokenStream {
    let variant_name_tokens = quote_escaped_ident(variant_name);
    let unbounded_param_tokens = quote_unbounded_params(params);
    let deserialize_where_tokens = quote_deserialize_where(params);
    let all_match_arms: Vec<_> =
        match_arms.iter().map(|variant| quote_try_from_trait_match_arm(variant_name, variant)).collect();
    let all_variant_types_string = match_arms.iter().map(|&field| field.name()).join(", ");
    quote!(
        impl #unbounded_param_tokens DamlDeserializeFrom for #variant_name_tokens #unbounded_param_tokens #deserialize_where_tokens {
            fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
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
fn quote_from_trait_match_arm(variant_name: &str, variant: &DamlField<'_>) -> TokenStream {
    let variant_name_tokens = quote_escaped_ident(variant_name);
    let name = quote_escaped_ident(variant.name());
    let variant_string = variant.name();
    if matches!(variant.ty(), DamlType::Unit) {
        quote!(
            #variant_name_tokens::#name => DamlValue::new_variant(DamlVariant::new(#variant_string, Box::new(DamlValue::new_unit()), None))
        )
    } else {
        let variant_type_tokens = quote_type(variant.ty());
        let serialize_value_tokens = quote!(
            <#variant_type_tokens as DamlSerializeInto<DamlValue>>::serialize_into(value)
        );
        quote!(
            #variant_name_tokens::#name(value) => DamlValue::new_variant(DamlVariant::new(#variant_string, Box::new(#serialize_value_tokens), None))
        )
    }
}

/// Quote a match arm of the `TryFrom<DamlValue> for Foo` `impl` block.
///
/// `"Variant" => Ok(VariantName::Variant(...))`
fn quote_try_from_trait_match_arm(variant_name: &str, variant: &DamlField<'_>) -> TokenStream {
    let variant_name_tokens = quote_escaped_ident(variant_name);
    let variant_constructor_name_tokens = quote_escaped_ident(variant.name());
    let variant_constructor_string = variant.name();
    let variant_type_tokens = quote_type(variant.ty());
    if matches!(variant.ty(), DamlType::Unit) {
        quote!(
            #variant_constructor_string => Ok(#variant_name_tokens::#variant_constructor_name_tokens)
        )
    } else {
        quote!(
            #variant_constructor_string => Ok(#variant_name_tokens::#variant_constructor_name_tokens(<#variant_type_tokens>::deserialize_from(*variant.take_value())?))
        )
    }
}

fn quote_unused_phantom_params(params: &[DamlTypeVarWithKind<'_>], variants: &[&DamlField<'_>]) -> TokenStream {
    let unused_params: Vec<_> = params
        .iter()
        .filter_map(|p| {
            variants.iter().any(|&f| f.ty().contains_type_var(p.var())).not().then(|| {
                let param_tokens = quote_ident(normalize_generic_param(p.var()).to_uppercase());
                quote!( std::marker::PhantomData < #param_tokens > )
            })
        })
        .collect();
    if unused_params.is_empty() {
        quote!()
    } else {
        quote!( _UnsupportedTypes ( #( #unused_params ),* ) )
    }
}
