use proc_macro2::{Ident, Span};
use syn::parse::Result;
use syn::parse::{Parse, ParseStream};
use syn::{Error, LitStr, Token};

#[derive(Debug)]
pub struct DamlTemplateInfo {
    pub package_id: String,
    pub module_name: String,
}

impl Parse for DamlTemplateInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        // parse 'package_id'
        match input.parse::<Ident>() {
            Ok(ref label) if label == "package_id" => Ok(()),
            Ok(ref label) => Err(Error::new(Span::call_site(), format!("expected 'package_id', found: '{}'", label))),
            Err(e) => Err(e),
        }?;

        // parse '='
        input.parse::<Token![=]>()?;

        // parse a string literal (the 'package_id')
        let package_id: LitStr = input.parse()?;

        // parse ','
        input.parse::<Token![,]>()?;

        // parse 'module_name'
        match input.parse::<Ident>() {
            Ok(ref label) if label == "module_name" => Ok(()),
            Ok(ref label) => Err(Error::new(Span::call_site(), format!("expected 'module_name', found: '{}'", label))),
            Err(e) => Err(e),
        }?;

        // parse '='
        input.parse::<Token![=]>()?;

        // parse a string literal (the 'module_name')
        let module_name: LitStr = input.parse()?;
        Ok(Self {
            package_id: package_id.value(),
            module_name: module_name.value(),
        })
    }
}
