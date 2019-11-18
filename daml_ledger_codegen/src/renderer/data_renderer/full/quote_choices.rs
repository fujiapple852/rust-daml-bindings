use proc_macro2::TokenStream;

use quote::quote;

use crate::element::{DamlChoice, DamlField, DamlType};
use crate::renderer::data_renderer::full::quote_contract_struct::quote_contract_id_struct_name;
use crate::renderer::expression_renderer::{quote_method_arguments, quote_new_value_expression, quote_try_expression};
use crate::renderer::is_supported_type;
use crate::renderer::renderer_utils::quote_escaped_ident;
use crate::renderer::type_renderer::quote_type;
use heck::SnakeCase;

pub fn quote_choice(name: &str, items: &[DamlChoice]) -> TokenStream {
    let all_choice_methods_tokens = quote_all_choice_methods(&name, items);
    let contract_struct_name_tokens = quote_contract_id_struct_name(name);
    quote!(
        impl #contract_struct_name_tokens {
            #all_choice_methods_tokens
        }
    )
}

/// Generate all choice methods within the parent `impl` block.
#[allow(clippy::filter_map)]
fn quote_all_choice_methods(struct_name: &str, items: &[DamlChoice]) -> TokenStream {
    let all_choice_methods: Vec<_> = items.iter().map(|choice| quote_choice_method(&struct_name, &choice)).collect();
    quote!(
        #( #all_choice_methods )*
    )
}

/// Generate the `pub fn foo(&self, ...)` choice method.
fn quote_choice_method(struct_name: &str, choice: &DamlChoice) -> TokenStream {
    let choice_name = &choice.name;
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let method_name_command_tokens = quote_command_method_name(&choice.name.to_snake_case());
    let method_name_tokens = quote_escaped_ident(&choice.name.to_snake_case());
    let choice_argument_tokens = quote_method_arguments(&choice.fields.iter().collect::<Vec<_>>());
    let supported_fields: Vec<_> = choice.fields.iter().filter(|&field| is_supported_type(&field.ty)).collect();
    let all_choice_fields = quote_all_choice_fields(&supported_fields);
    let return_type_tokens = quote_type(&choice.return_type);
    let try_return_type_expression_tokens = quote_try_expression(&choice.return_type);
    quote!(
        pub fn #method_name_command_tokens(&self, #choice_argument_tokens) -> DamlExerciseCommand {
            let template_id = #struct_name_tokens::package_id();
            #all_choice_fields
            DamlExerciseCommand::new(
                template_id,
                self.contract_id(),
                #choice_name,
                params
            )
        }
        pub fn #method_name_tokens<E: CommandExecutor>(&self, #choice_argument_tokens) -> impl FnOnce(&E) -> DamlResult<#return_type_tokens> + '_ {
            let template_id = #struct_name_tokens::package_id();
            #all_choice_fields
            let exercise_command = DamlExerciseCommand::new(
                template_id,
                self.contract_id(),
                #choice_name,
                params
            );
            move |exec| {
                let value = exec.execute_exercise(exercise_command)?;
                #try_return_type_expression_tokens
            }
        }
    )
}

/// Generate the `DamlValue::Record` containing all choice fields.
fn quote_all_choice_fields(supported_fields: &[&DamlField]) -> TokenStream {
    let all_choice_fields = quote_declare_all_choice_fields(supported_fields);
    if all_choice_fields.is_empty() {
        quote!(
            let params = DamlValue::Record(DamlRecord::new(vec![], None::<DamlIdentifier>));
        )
    } else {
        quote!(
            let mut records = vec![];
            #all_choice_fields
            let params = DamlValue::Record(DamlRecord::new(records, None::<DamlIdentifier>));
        )
    }
}

/// Generate all choice fields.
fn quote_declare_all_choice_fields(choice_parameters: &[&DamlField]) -> TokenStream {
    choice_parameters
        .iter()
        .map(
            |DamlField {
                 name,
                 ty,
             }| quote_declare_choice_field(name, ty),
        )
        .collect()
}

/// Generate a choice field.
fn quote_declare_choice_field(field_name: &str, field_type: &DamlType) -> TokenStream {
    let field_name = quote_escaped_ident(field_name);
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

fn quote_command_method_name(name: &str) -> TokenStream {
    quote_escaped_ident(format!("{}_command", name))
}
