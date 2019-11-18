use syn::{Data, DataStruct, DeriveInput, Fields};

use crate::convert::attribute::attr_element::{extract_template, AttrTemplate};
use crate::element::DamlTemplate;
use crate::renderer::full::quote_daml_template;

pub fn generate_template(input: DeriveInput, package_id: String, module_name: String) -> proc_macro::TokenStream {
    let struct_name = input.ident.to_string();
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields_named),
            ..
        }) => {
            let template: AttrTemplate = extract_template(struct_name, package_id, module_name, fields_named);
            let daml_template = DamlTemplate::from(&template);
            let expanded = quote_daml_template(&daml_template);
            proc_macro::TokenStream::from(expanded)
        },
        _ => panic!("the DamlTemplate attribute may only be applied to a named struct type"),
    }
}
