use proc_macro2::TokenStream;

use quote::quote;

use crate::element::{DamlField, DamlRecord, DamlType};
use crate::renderer::expression_renderer::{quote_method_arguments, quote_new_value_expression, quote_try_expression};
use crate::renderer::type_renderer::quote_type;
use crate::renderer::{is_supported_type, quote_escaped_ident};

/// Generate the `Foo` struct and methods.
pub fn quote_daml_record(daml_record: &DamlRecord) -> TokenStream {
    quote_daml_record_and_impl(daml_record.name, &daml_record.fields)
}

/// Generate the `Foo` struct and methods.
pub fn quote_daml_record_and_impl(name: &str, fields: &[DamlField]) -> TokenStream {
    let supported_fields: Vec<_> = fields.iter().filter(|&field| is_supported_type(&field.ty)).collect();
    let struct_tokens = quote_struct(name, &supported_fields);
    let new_method_tokens = quote_new_method(name, &supported_fields);
    let from_trait_impl_tokens = quote_from_trait_impl(name, &supported_fields);
    let try_from_trait_impl_tokens = quote_try_from_trait_impl(name, &supported_fields);
    quote!(
        #struct_tokens
        #new_method_tokens
        #from_trait_impl_tokens
        #try_from_trait_impl_tokens
    )
}

/// Generate `struct Foo {...}` struct.
fn quote_struct(struct_name: &str, struct_fields: &[&DamlField]) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let body_tokens = quote_struct_body(struct_fields);
    quote!(
        #[derive(Eq, PartialEq, Clone, Debug)]
        pub struct #struct_name_tokens {
            #body_tokens
        }
    )
}

/// Generate the `Foo::new(...)` method.
fn quote_new_method(struct_name: &str, struct_fields: &[&DamlField]) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let method_arguments_tokens = quote_method_arguments(struct_fields);
    let method_body_tokens = quote_new_method_init(struct_fields);
    quote! {
        impl #struct_name_tokens {
            pub fn new( #method_arguments_tokens ) -> Self {
                Self {
                    #method_body_tokens
                }
            }
        }
    }
}

/// Generate the `From<Foo> for DamlValue` method.
fn quote_from_trait_impl(struct_name: &str, struct_fields: &[&DamlField]) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let all_fields: Vec<_> = struct_fields
        .iter()
        .map(
            |DamlField {
                 name,
                 ty,
             }| quote_declare_field_from_trait_impl(name, ty),
        )
        .collect();

    if all_fields.is_empty() {
        quote! {
            impl From<#struct_name_tokens> for DamlValue {
                fn from(_: #struct_name_tokens) -> Self {
                    DamlValue::Record(DamlRecord::new(vec![], None::<DamlIdentifier>))
                }
            }
        }
    } else {
        quote! {
            impl From<#struct_name_tokens> for DamlValue {
                fn from(value: #struct_name_tokens) -> Self {
                    let mut records = vec![];
                    #( #all_fields )*
                    DamlValue::Record(DamlRecord::new(records, None::<DamlIdentifier>))
                }
            }
        }
    }
}

/// Generate the `TryFrom<DamlValue> for Foo` method.
fn quote_try_from_trait_impl(struct_name: &str, struct_fields: &[&DamlField]) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let all_fields: Vec<_> = struct_fields.iter().map(|&f| quote_try_from_trait_field(f)).collect();
    if all_fields.is_empty() {
        quote!(
            impl TryFrom<DamlValue> for #struct_name_tokens {
                type Error = DamlError;
                fn try_from(_: DamlValue) -> std::result::Result<Self, <#struct_name_tokens as TryFrom<DamlValue>>::Error> {
                    Ok(Self::new())
                }
            }
        )
    } else {
        quote!(
            impl TryFrom<DamlValue> for #struct_name_tokens {
                type Error = DamlError;
                fn try_from(value: DamlValue) -> std::result::Result<Self, <#struct_name_tokens as TryFrom<DamlValue>>::Error> {
                    let record = value.try_record()?;
                    Ok(Self::new(
                        #( #all_fields ),*
                    ))
                }
            }
        )
    }
}

fn quote_struct_body(struct_fields: &[&DamlField]) -> TokenStream {
    let all: Vec<_> = struct_fields
        .iter()
        .map(
            |DamlField {
                 name,
                 ty,
             }| {
                let field_label = quote_escaped_ident(name);
                let field_type_rendered = quote_type(ty);
                quote!(pub #field_label: #field_type_rendered)
            },
        )
        .collect();
    quote!( #( #all ,)* )
}

fn quote_new_method_init(struct_fields: &[&DamlField]) -> TokenStream {
    let all: Vec<_> = struct_fields
        .iter()
        .map(
            |DamlField {
                 name,
                 ..
             }| {
                let field_label = quote_escaped_ident(name);
                quote!(#field_label: #field_label.into())
            },
        )
        .collect();
    quote!( #( #all ,)* )
}

fn quote_declare_field_from_trait_impl(field_name: &str, field_type: &DamlType) -> TokenStream {
    let field_name_tokens = quote_escaped_ident(field_name);
    let field_source_tokens = quote!(value.#field_name_tokens);
    let rendered_new_value_tokens = quote_new_value_expression(field_type);
    let name_string = quote!(stringify!(#field_name_tokens));
    quote!(
        {
            let #field_name_tokens: DamlValue = {
                let value = #field_source_tokens;
                #rendered_new_value_tokens
            };
            records.push(DamlRecordField::new(Some(#name_string), #field_name_tokens));
        }
    )
}

fn quote_try_from_trait_field(field: &DamlField) -> TokenStream {
    let field_name_string = &field.name;
    let try_field_expression_tokens = quote_try_expression(&field.ty);
    quote!(
        {
            let value = record.field(#field_name_string)?.to_owned();
            #try_field_expression_tokens?
        }
    )
}
