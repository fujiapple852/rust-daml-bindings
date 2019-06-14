use crate::daml_element::data_type_string_from_type;
use crate::quote::{quote_all_choice_methods, quote_contract_struct_name};
use quote::quote;
use syn::ItemImpl;

pub fn generate_choices(input: ItemImpl) -> proc_macro::TokenStream {
    let struct_name = data_type_string_from_type(input.self_ty.as_ref());
    let contract_struct_name_tokens = quote_contract_struct_name(&struct_name);
    let all_choice_methods_tokens = quote_all_choice_methods(&struct_name, &input.items);
    let expanded = quote!(
        impl #contract_struct_name_tokens {
            #all_choice_methods_tokens
        }
    );
    proc_macro::TokenStream::from(expanded)
}
