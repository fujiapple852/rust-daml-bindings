use proc_macro2::TokenStream;

use quote::quote;

use crate::element::DamlTemplate;

use crate::renderer::data_renderer::full::quote_contract_struct::{
    quote_contract_struct_and_impl, quote_contract_struct_name,
};
use crate::renderer::data_renderer::full::{quote_choice, quote_daml_record_and_impl};
use crate::renderer::{quote_escaped_ident, to_module_path};

pub fn quote_daml_template(daml_template: &DamlTemplate) -> TokenStream {
    let struct_and_impl_tokens = quote_daml_record_and_impl(&daml_template.name, &daml_template.fields);
    let package_id_method_tokens =
        quote_package_id_method(&daml_template.name, &daml_template.package_id, &daml_template.module_path);
    let make_create_method_tokens = quote_make_create_command_method(&daml_template.name);
    let contract_struct_and_impl_tokens = quote_contract_struct_and_impl(&daml_template.name);
    let choices_impl_tokens = quote_choice(&daml_template.name, &daml_template.choices);
    quote!(
        #struct_and_impl_tokens
        #package_id_method_tokens
        #make_create_method_tokens
        #contract_struct_and_impl_tokens
        #choices_impl_tokens
    )
}

/// Generate the `pub fn package_id(...) -> DamlIdentifier` method.
pub fn quote_package_id_method(struct_name: &str, package_id: &str, path: &[&str]) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let package_id = package_id;
    let module_name = to_module_path(path);
    let entity_name = struct_name;
    quote!(
        impl #struct_name_tokens {
            pub fn package_id() -> DamlIdentifier {
                DamlIdentifier::new(#package_id, #module_name, #entity_name)
            }
        }
    )
}

/// Generate the `pub fn create(...) & pub fn create_command()` methods.
pub fn quote_make_create_command_method(struct_name: &str) -> TokenStream {
    let struct_name_tokens = quote_escaped_ident(struct_name);
    let _contract_struct_name = quote_contract_struct_name(struct_name);

    // TODO restore
    // pub fn create<E: CommandExecutor>(&self) -> impl FnOnce(&E) -> DamlResult<#contract_struct_name> + '_ {
    //        let template_id = Self::package_id();
    //        let value: DamlValue = self.to_owned().into();
    //        let create_command = DamlCreateCommand::new(template_id, value.try_take_record().unwrap());
    //    move |exec| {
    //        let result = exec.execute_create(create_command)?;
    //        #contract_struct_name::try_from(result)
    //    }
    //}

    quote!(
        impl #struct_name_tokens {
            pub fn create_command(&self) -> DamlCreateCommand {
                let template_id = Self::package_id();
                let value: DamlValue = self.to_owned().into();
                DamlCreateCommand::new(template_id, value.try_take_record().unwrap())
            }
        }
    )
}
