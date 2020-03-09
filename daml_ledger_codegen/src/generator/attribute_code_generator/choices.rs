use syn::ItemImpl;

use crate::convert::attribute::attr_element::{data_type_string_from_type, extract_all_choices, AttrChoice};
use crate::element::DamlChoice;
use crate::renderer::full::quote_choice;

pub fn generate_choices(input: ItemImpl) -> proc_macro::TokenStream {
    let struct_name = data_type_string_from_type(input.self_ty.as_ref());
    let all_choices: Vec<AttrChoice> = extract_all_choices(&input.items);
    let all_daml_choices: Vec<DamlChoice<'_>> = all_choices.iter().map(DamlChoice::from).collect();
    let all_choice_methods_tokens = quote_choice(&struct_name, &all_daml_choices);
    proc_macro::TokenStream::from(all_choice_methods_tokens)
}
