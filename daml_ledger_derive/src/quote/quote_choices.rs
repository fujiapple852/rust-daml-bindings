use crate::daml_element::{get_single_attr_method, DamlChoice};
use crate::daml_element::{DamlField, DamlType};
use crate::quote::{quote_ident, quote_method_arguments, quote_new_value_expression};
use proc_macro2::TokenStream;
use quote::quote;
use syn::ImplItem;

/// Generate all choice methods within the parent `impl` block.
#[allow(clippy::filter_map)]
pub fn quote_all_choice_methods(struct_name: &str, items: &[ImplItem]) -> TokenStream {
    let all_choice_methods: Vec<_> = items
        .iter()
        .filter_map(get_single_attr_method)
        .map(|choice| quote_choice_method(&struct_name, &choice))
        .collect();
    quote!(
        #( #all_choice_methods )*
    )
}

/// Generate the `pub fn foo(&self, ...)` method.
fn quote_choice_method(struct_name: &str, choice: &DamlChoice) -> TokenStream {
    let choice_name = &choice.choice_name;
    let struct_name_tokens = quote_ident(struct_name);
    let method_name_tokens = quote_ident(&choice.choice_method_name);
    let choice_argument_tokens = quote_method_arguments(&choice.choice_arguments);
    let all_choice_fields = quote_declare_all_choice_fields(&choice.choice_arguments);
    quote!(
        pub fn #method_name_tokens(&self, #choice_argument_tokens) -> DamlCommand {
            let template_id = #struct_name_tokens::package_id();
            let mut records = vec![];
            #all_choice_fields
            let params = DamlValue::Record(DamlRecord::new(records, None::<DamlIdentifier>));
            DamlCommand::Exercise(
                DamlExerciseCommand::new(
                    template_id,
                    self.id(),
                    #choice_name,
                    params
                )
            )
        }
    )
}

/// Generate all choice fields.
fn quote_declare_all_choice_fields(choice_parameters: &[DamlField]) -> TokenStream {
    choice_parameters
        .iter()
        .map(
            |DamlField {
                 field_label,
                 field_type,
             }| quote_declare_choice_field(field_label, field_type),
        )
        .collect()
}

/// Generate a choice field.
fn quote_declare_choice_field(field_name: &str, field_type: &DamlType) -> TokenStream {
    let field_name = quote_ident(field_name);
    let field_source_tokens = quote!(#field_name);
    let rendered_new_value_tokens = quote_new_value_expression(field_type);
    let name_string = quote!(stringify!(#field_name));
    quote!(
        let #field_name: DamlValue = {
            let value = #field_source_tokens.into();
            #rendered_new_value_tokens
        };
        records.push(DamlRecordField::new(Some(#name_string), #field_name));
    )
}
