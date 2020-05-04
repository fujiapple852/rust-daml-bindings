use proc_macro2::TokenStream;

use quote::quote;

use crate::renderer::data_renderer::full::{
    quote_bounded_params, quote_deserialize_where, quote_method_arguments, quote_serialize_where,
    quote_unbounded_params,
};
use crate::renderer::render_context::RenderContext;
use crate::renderer::type_renderer::quote_type;
use crate::renderer::{make_ignored_ident, normalize_generic_param, quote_escaped_ident, quote_ident, IsRenderable};
use daml_lf::element::{DamlField, DamlRecord, DamlType, DamlTypeVar};

/// Generate the `Foo` struct and methods.
pub fn quote_daml_record(ctx: &RenderContext<'_>, daml_record: &DamlRecord<'_>) -> TokenStream {
    quote_daml_record_and_impl(ctx, daml_record.name(), daml_record.fields(), daml_record.type_arguments())
}

/// Generate the `Foo` struct and methods.
pub fn quote_daml_record_and_impl(
    ctx: &RenderContext<'_>,
    name: &str,
    fields: &[DamlField<'_>],
    params: &[DamlTypeVar<'_>],
) -> TokenStream {
    let supported_fields: Vec<_> =
        fields.iter().filter(|&field| IsRenderable::new(ctx).check_type(field.ty())).collect();
    let struct_tokens = quote_struct(name, &supported_fields, params);
    let new_method_tokens = quote_new_method(name, &supported_fields, params);
    let serialize_trait_impl_tokens = quote_serialize_trait_impl(name, &supported_fields, params);
    let deserialize_trait_impl_tokens = quote_deserialize_trait_impl(name, &supported_fields, params);
    quote!(
        #struct_tokens
        #new_method_tokens
        #serialize_trait_impl_tokens
        #deserialize_trait_impl_tokens
    )
}

/// Generate `struct Foo {...}` struct.
fn quote_struct(struct_name: &str, struct_fields: &[&DamlField<'_>], params: &[DamlTypeVar<'_>]) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let bounded_param_tokens = quote_bounded_params(params);
    let unbounded_param_tokens = quote_unbounded_params(params);
    let body_tokens = quote_struct_body(struct_fields);
    let phantom_tokens = quote_decl_unused_phantom_params(params, struct_fields);
    quote!(
        #[derive(Eq, PartialEq, Clone, Debug)]
        pub struct #struct_name_tokens #bounded_param_tokens {
            #body_tokens
            #phantom_tokens
        }
        impl #bounded_param_tokens DamlDeserializableType for #struct_name_tokens #unbounded_param_tokens {}
        impl #bounded_param_tokens DamlSerializableType for #struct_name_tokens #unbounded_param_tokens {}
    )
}

/// Generate the `Foo::new(...)` method.
fn quote_new_method(struct_name: &str, struct_fields: &[&DamlField<'_>], params: &[DamlTypeVar<'_>]) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let method_arguments_tokens = quote_method_arguments(struct_fields);
    let method_body_tokens = quote_new_method_init(struct_fields);
    let bounded_param_tokens = quote_bounded_params(params);
    let unbounded_param_tokens = quote_unbounded_params(params);
    let method_phantom_tokens = quote_init_unused_phantom_params(params, struct_fields);
    quote! {
        impl #bounded_param_tokens #struct_name_tokens #unbounded_param_tokens {
            pub fn new( #method_arguments_tokens ) -> Self {
                Self {
                    #method_body_tokens
                    #method_phantom_tokens
                }
            }
        }
    }
}

/// Generate the `DamlSerializeFrom<Foo> for DamlValue` method.
fn quote_serialize_trait_impl(
    struct_name: &str,
    struct_fields: &[&DamlField<'_>],
    params: &[DamlTypeVar<'_>],
) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let unbounded_param_tokens = quote_unbounded_params(params);
    let serialize_where_tokens = quote_serialize_where(params);
    let body_tokens = quote_serialize_impl_body(struct_fields);
    quote!(
        impl #unbounded_param_tokens DamlSerializeFrom<#struct_name_tokens #unbounded_param_tokens> for DamlValue #serialize_where_tokens {
            fn serialize_from(value: #struct_name_tokens #unbounded_param_tokens) -> Self {
                #body_tokens
            }
        }
    )
}

/// Generate the `DamlDeserializeFrom for Foo` method.
fn quote_deserialize_trait_impl(
    struct_name: &str,
    struct_fields: &[&DamlField<'_>],
    params: &[DamlTypeVar<'_>],
) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let unbounded_param_tokens = quote_unbounded_params(params);
    let deserialize_where_tokens = quote_deserialize_where(params);
    let body_tokens = quote_deserialize_trait_impl_body(struct_fields);
    quote!(
        impl #unbounded_param_tokens DamlDeserializeFrom for #struct_name_tokens #unbounded_param_tokens #deserialize_where_tokens {
            fn deserialize_from(value: DamlValue) -> DamlResult<Self> {
                #body_tokens
            }
        }
    )
}

fn quote_struct_body(struct_fields: &[&DamlField<'_>]) -> TokenStream {
    let all: Vec<_> = struct_fields
        .iter()
        .map(|&field| {
            let field_label = quote_escaped_ident(field.name());
            let field_type_rendered = quote_type(field.ty());
            quote!(pub #field_label: #field_type_rendered)
        })
        .collect();
    quote!( #( #all ,)* )
}

fn quote_new_method_init(struct_fields: &[&DamlField<'_>]) -> TokenStream {
    let all: Vec<_> = struct_fields
        .iter()
        .map(|&field| {
            let field_label = quote_escaped_ident(field.name());
            quote!(#field_label: #field_label.into())
        })
        .collect();
    quote!( #( #all ,)* )
}

fn quote_serialize_impl_body(struct_fields: &[&DamlField<'_>]) -> TokenStream {
    if struct_fields.is_empty() {
        quote!(DamlValue::Record(DamlRecord::new(vec![], None::<DamlIdentifier>)))
    } else {
        let all_fields: Vec<_> = struct_fields
            .iter()
            .map(|&field| quote_declare_field_serialize_trait_impl(field.name(), field.ty()))
            .collect();
        quote!(DamlValue::Record(DamlRecord::new(vec![#( #all_fields ),*], None::<DamlIdentifier>)))
    }
}

fn quote_deserialize_trait_impl_body(struct_fields: &[&DamlField<'_>]) -> TokenStream {
    if struct_fields.is_empty() {
        quote!(Ok(Self::new()))
    } else {
        let all_fields: Vec<_> = struct_fields.iter().map(|&f| quote_deserialize_trait_field(f)).collect();
        quote!(
            let record = value.try_record()?;
            Ok(Self::new(
                #( #all_fields ),*
            ))
        )
    }
}

fn quote_declare_field_serialize_trait_impl(field_name: &str, ty: &DamlType<'_>) -> TokenStream {
    let field_name_tokens = quote_escaped_ident(field_name);
    let field_source_tokens = quote!(value.#field_name_tokens);
    let field_type_tokens = quote_type(ty);
    quote!(
        DamlRecordField::new(Some(#field_name), <#field_type_tokens as DamlSerializeInto<DamlValue>>::serialize_into(#field_source_tokens))
    )
}

fn quote_deserialize_trait_field(field: &DamlField<'_>) -> TokenStream {
    let field_name_string = field.name();
    let field_type_tokens = quote_type(field.ty());
    quote!(
        <#field_type_tokens>::deserialize_from(record.field(#field_name_string)?.to_owned())?
    )
}

fn quote_decl_unused_phantom_params(params: &[DamlTypeVar<'_>], struct_fields: &[&DamlField<'_>]) -> TokenStream {
    quote_unused_phantom_params(params, struct_fields, |param| {
        let name_tokens = quote_escaped_ident(make_ignored_ident(param.var()));
        let param_tokens = quote_ident(normalize_generic_param(param.var()).to_uppercase());
        quote!( #name_tokens: std::marker::PhantomData< #param_tokens >)
    })
}

fn quote_init_unused_phantom_params(params: &[DamlTypeVar<'_>], struct_fields: &[&DamlField<'_>]) -> TokenStream {
    quote_unused_phantom_params(params, struct_fields, |param| {
        let name_tokens = quote_escaped_ident(make_ignored_ident(param.var()));
        quote!( #name_tokens: std::marker::PhantomData )
    })
}

fn quote_unused_phantom_params(
    params: &[DamlTypeVar<'_>],
    struct_fields: &[&DamlField<'_>],
    type_var_quoter: impl Fn(&DamlTypeVar<'_>) -> TokenStream,
) -> TokenStream {
    let all_params: Vec<_> = params
        .iter()
        .filter_map(|p| {
            if struct_fields.iter().any(|&f| f.ty().contains_type_var(p.var())) {
                None
            } else {
                Some(type_var_quoter(p))
            }
        })
        .collect();
    quote!( #( #all_params ),* )
}
