use crate::daml_element::DamlTemplateInfo;
use crate::quote::quote_ident;
use proc_macro2::TokenStream;
use quote::quote;

/// Generate the `pub fn package_id(...) -> DamlIdentifier` method.
pub fn quote_package_id_method(struct_name: &str, template_info: &DamlTemplateInfo) -> TokenStream {
    let struct_name_tokens = quote_ident(struct_name);
    let package_id = &template_info.package_id;
    let module_name = &template_info.module_name;
    let entity_name = struct_name;
    quote!(
        impl #struct_name_tokens {
            pub fn package_id() -> DamlIdentifier {
                DamlIdentifier::new(#package_id, #module_name, #entity_name)
            }
        }
    )
}

/// Generate the `pub fn make_create(...) -> DamlCommand` method.
pub fn quote_make_create_command_method(struct_name: &str) -> TokenStream {
    let struct_name_tokens = quote_ident(struct_name);
    quote!(
        impl #struct_name_tokens {
            pub fn create(&self) -> DamlCommand {
                let template_id = Self::package_id();
                let value: DamlValue = self.to_owned().into();
                DamlCommand::Create(
                    DamlCreateCommand::new(template_id, value.try_take_record().expect("impossible failure"))
                )
            }
        }
    )
}
