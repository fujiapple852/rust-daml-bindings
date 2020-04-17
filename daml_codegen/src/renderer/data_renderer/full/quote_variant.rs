use itertools::Itertools;
use proc_macro2::TokenStream;

use quote::quote;

use crate::renderer::data_renderer::full::{
    quote_deserialize_generic_trait_bounds, quote_generic_param_list, quote_serialize_generic_trait_bounds,
};
use crate::renderer::type_renderer::quote_type;
use crate::renderer::{is_supported_type, normalize_generic_param, quote_escaped_ident, quote_ident};
use daml_lf::element::{DamlField, DamlType, DamlTypeVar, DamlVariant};

/// Generate the variant `enum` and the `DamlDeserializeFrom` and `DamlDeserializeFrom` impls.
pub fn quote_daml_variant(variant: &DamlVariant<'_>) -> TokenStream {
    let supported_fields: Vec<_> = variant.fields.iter().filter(|&field| is_supported_type(&field.ty)).collect();
    let variant_tokens = quote_variant(variant.name, &supported_fields, &variant.type_arguments);
    let serialize_trait_impl_tokens =
        quote_serialize_trait_impl(variant.name, &supported_fields, &variant.type_arguments);
    let deserialize_trait_impl_tokens =
        quote_deserialize_trait_impl(variant.name, &supported_fields, &variant.type_arguments);
    quote!(
        #variant_tokens
        #serialize_trait_impl_tokens
        #deserialize_trait_impl_tokens
    )
}

/// Generate `enum Foo {...}` variant.
fn quote_variant(variant_name: &str, variants: &[&DamlField<'_>], params: &[DamlTypeVar<'_>]) -> TokenStream {
    let enum_name_tokens = quote_escaped_ident(variant_name);
    let generic_param_tokens = quote_generic_param_list(params);
    let body_tokens = quote_variant_body(variants);
    let phantom_tokens = quote_unused_phantom_params(params, variants);
    quote!(
        #[derive(Eq, PartialEq, Clone, Debug)]
        pub enum #enum_name_tokens #generic_param_tokens {
            #body_tokens
            #phantom_tokens
        }
        impl #generic_param_tokens DamlDeserializableType for #enum_name_tokens #generic_param_tokens {}
        impl #generic_param_tokens DamlSerializableType for #enum_name_tokens #generic_param_tokens {}
    )
}

/// Generate the variant body.
fn quote_variant_body(variants: &[&DamlField<'_>]) -> TokenStream {
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

/// Generate the `DamlSerializeFrom<Foo> for DamlValue` method.
fn quote_serialize_trait_impl(
    variant_name: &str,
    variants: &[&DamlField<'_>],
    params: &[DamlTypeVar<'_>],
) -> TokenStream {
    let variant_name_tokens = quote_escaped_ident(variant_name);
    let generic_param_tokens = quote_generic_param_list(params);
    let generic_trail_bounds_tokens = quote_serialize_generic_trait_bounds(params);
    let all_match_arms: Vec<_> =
        variants.iter().map(|variant| quote_from_trait_match_arm(variant_name, variant)).collect();
    quote! {
        impl #generic_param_tokens DamlSerializeFrom<#variant_name_tokens #generic_param_tokens> for DamlValue #generic_trail_bounds_tokens {
            fn serialize_from(value: #variant_name_tokens #generic_param_tokens) -> Self {
                match value {
                    #( #all_match_arms ,)*
                    _ => panic!(format!("type {} cannot be serialized", stringify!(#variant_name_tokens)))
                }
            }
        }
    }
}

/// Generate the `DamlDeserializeFrom for Foo` method.
fn quote_deserialize_trait_impl(
    variant_name: &str,
    match_arms: &[&DamlField<'_>],
    params: &[DamlTypeVar<'_>],
) -> TokenStream {
    let variant_name_tokens = quote_escaped_ident(variant_name);
    let generic_param_tokens = quote_generic_param_list(params);
    let generic_trail_bounds_tokens = quote_deserialize_generic_trait_bounds(params);
    let all_match_arms: Vec<_> =
        match_arms.iter().map(|variant| quote_try_from_trait_match_arm(variant_name, variant)).collect();
    let all_variant_types_string = match_arms
        .iter()
        .map(
            |DamlField {
                 name,
                 ..
             }| name,
        )
        .join(", ");
    quote!(
        impl #generic_param_tokens DamlDeserializeFrom for #variant_name_tokens #generic_param_tokens #generic_trail_bounds_tokens {
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
    let name = quote_escaped_ident(variant.name);
    let variant_string = variant.name;
    if let DamlType::Unit = variant.ty {
        quote!(
            #variant_name_tokens::#name => DamlValue::new_variant(DamlVariant::new(#variant_string, Box::new(DamlValue::new_unit()), None))
        )
    } else {
        let variant_type_tokens = quote_type(&variant.ty);
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
    let variant_constructor_name_tokens = quote_escaped_ident(variant.name);
    let variant_constructor_string = variant.name;
    let variant_type_tokens = quote_type(&variant.ty);
    if let DamlType::Unit = &variant.ty {
        quote!(
            #variant_constructor_string => Ok(#variant_name_tokens::#variant_constructor_name_tokens)
        )
    } else {
        quote!(
            #variant_constructor_string => Ok(#variant_name_tokens::#variant_constructor_name_tokens(<#variant_type_tokens>::deserialize_from(*variant.take_value())?))
        )
    }
}

fn quote_unused_phantom_params(params: &[DamlTypeVar<'_>], variants: &[&DamlField<'_>]) -> TokenStream {
    let unused_params: Vec<_> = params
        .iter()
        .filter_map(|p| {
            if variants.iter().any(|&f| f.ty.contains_type_var(p.var)) {
                None
            } else {
                Some({
                    let param_tokens = quote_ident(normalize_generic_param(p.var).to_uppercase());
                    quote!( std::marker::PhantomData < #param_tokens > )
                })
            }
        })
        .collect();
    if unused_params.is_empty() {
        quote!()
    } else {
        quote!( _UnsupportedTypes ( #( #unused_params ),* ) )
    }
}