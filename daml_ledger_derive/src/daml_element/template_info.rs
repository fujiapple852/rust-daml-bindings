use proc_macro2::Ident;
use std::collections::HashMap;
use syn::parse::Result;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};

#[derive(Debug)]
pub struct DamlTemplateInfo {
    pub package_id: String,
    pub module_name: String,
}

impl Parse for DamlTemplateInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        let all_properties: HashMap<String, String> = input
            .parse_terminated::<TemplateProperty, Token![,]>(TemplateProperty::parse)?
            .into_iter()
            .map(|prop| (prop.key, prop.value))
            .collect();
        match (all_properties.get("package_id"), all_properties.get("module_name")) {
            (Some(package_id), Some(module_name)) => Ok(Self {
                package_id: package_id.to_owned(),
                module_name: module_name.to_owned(),
            }),
            _ => panic!(format!("expected 'package_id' and 'module_name', found {:?}", all_properties.keys())),
        }
    }
}

#[derive(Debug)]
struct TemplateProperty {
    key: String,
    value: String,
}

impl Parse for TemplateProperty {
    fn parse(input: ParseStream) -> Result<Self> {
        let key = input.parse::<Ident>()?.to_string();
        input.parse::<Token![=]>()?;
        let value: LitStr = input.parse()?;
        Ok(Self {
            key,
            value: value.value(),
        })
    }
}
