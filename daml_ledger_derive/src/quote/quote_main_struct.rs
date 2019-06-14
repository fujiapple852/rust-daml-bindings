use crate::daml_element::{extract_struct_data, DamlField, DamlType};
use crate::quote::{quote_ident, quote_method_arguments, quote_new_value_expression, quote_try_expression, quote_type};
use proc_macro2::TokenStream;
use quote::quote;
use syn::FieldsNamed;

/// Generate the `Foo` struct and methods.
pub fn quote_struct_and_impl(name: &str, fields_named: &FieldsNamed) -> TokenStream {
    let struct_fields = extract_struct_data(fields_named);
    let struct_tokens = quote_struct(&name, &struct_fields);
    let new_method_tokens = quote_new_method(&name, &struct_fields);
    let from_trait_impl_tokens = quote_from_trait_impl(&name, &struct_fields);
    let try_from_trait_impl_tokens = quote_try_from_trait_impl(&name, &struct_fields);
    quote!(
        #struct_tokens
        #new_method_tokens
        #from_trait_impl_tokens
        #try_from_trait_impl_tokens
    )
}

/// Generate `struct Foo {...}` struct.
fn quote_struct(struct_name: &str, struct_fields: &[DamlField]) -> TokenStream {
    let struct_name_tokens = quote_ident(struct_name);
    let body_tokens = quote_struct_body(&struct_fields);
    quote!(
        #[derive(Eq, PartialEq, Clone, Debug)]
        pub struct #struct_name_tokens {
            #body_tokens
        }
    )
}

/// Generate the `Foo::new(...)` method.
fn quote_new_method(struct_name: &str, struct_fields: &[DamlField]) -> TokenStream {
    let struct_name_tokens = quote_ident(struct_name);
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
fn quote_from_trait_impl(struct_name: &str, struct_fields: &[DamlField]) -> TokenStream {
    let struct_name_tokens = quote_ident(struct_name);
    let all_fields: Vec<_> = struct_fields
        .iter()
        .map(
            |DamlField {
                 field_label,
                 field_type,
             }| quote_declare_field_from_trait_impl(field_label, field_type),
        )
        .collect();
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

/// Generate the `TryFrom<DamlValue> for Foo` method.
fn quote_try_from_trait_impl(struct_name: &str, struct_fields: &[DamlField]) -> TokenStream {
    let struct_name_tokens = quote_ident(struct_name);
    let all_fields: Vec<_> = struct_fields.iter().map(quote_try_from_trait_field).collect();
    quote!(
        impl TryFrom<DamlValue> for #struct_name_tokens {
            type Error = DamlError;
            fn try_from(value: DamlValue) -> Result<Self, Self::Error> {
                let record = value.try_record()?;
                Ok(Self::new(
                    #( #all_fields ),*
                ))
            }
        }
    )
}

fn quote_struct_body(struct_fields: &[DamlField]) -> TokenStream {
    let all: Vec<_> = struct_fields
        .iter()
        .map(
            |DamlField {
                 field_label,
                 field_type,
             }| {
                let field_label = quote_ident(field_label);
                let field_type_rendered = quote_type(field_type);
                quote!(pub #field_label: #field_type_rendered)
            },
        )
        .collect();
    quote!( #( #all ,)* )
}

fn quote_new_method_init(struct_fields: &[DamlField]) -> TokenStream {
    let all: Vec<_> = struct_fields
        .iter()
        .map(
            |DamlField {
                 field_label,
                 ..
             }| {
                let field_label = quote_ident(field_label);
                quote!(#field_label: #field_label.into())
            },
        )
        .collect();
    quote!( #( #all ,)* )
}

fn quote_declare_field_from_trait_impl(field_name: &str, field_type: &DamlType) -> TokenStream {
    let field_name_tokens = quote_ident(field_name);
    let field_source_tokens = quote!(value.#field_name_tokens);
    let rendered_new_value_tokens = quote_new_value_expression(field_type);
    let name_string = quote!(stringify!(#field_name_tokens));
    quote!(
        let #field_name_tokens: DamlValue = {
            let value = #field_source_tokens;
            #rendered_new_value_tokens
        };
        records.push(DamlRecordField::new(Some(#name_string), #field_name_tokens));
    )
}

fn quote_try_from_trait_field(field: &DamlField) -> TokenStream {
    let field_name_string = &field.field_label;
    let try_field_expression_tokens = quote_try_expression(&field.field_type);
    quote!(
        {
            let value = record.field(#field_name_string)?.to_owned();
            #try_field_expression_tokens?
        }
    )
}
