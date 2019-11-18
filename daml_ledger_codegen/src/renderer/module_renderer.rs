use heck::SnakeCase;
use proc_macro2::TokenStream;

use quote::quote;

use crate::element::*;
use crate::generator::ModuleMatcher;
use crate::generator::RenderMethod;
use crate::renderer::{quote_all_data, quote_escaped_ident, to_module_path};

pub fn quote_module_tree(
    name: &str,
    module: &DamlModule,
    module_matcher: &ModuleMatcher,
    render_method: &RenderMethod,
) -> TokenStream {
    let is_included_module = module_matcher.matches(&to_module_path(module.path));
    let all_children: Vec<_> = module
        .child_modules
        .values()
        .map(|child| quote_module_tree(child.name, child, module_matcher, render_method))
        .collect();
    let all_empty_children = all_children.iter().all(TokenStream::is_empty);
    if !is_included_module && all_empty_children {
        quote!()
    } else {
        let module_tokens = if is_included_module {
            quote_all_data(module.data_types.values().collect::<Vec<_>>().as_slice(), render_method)
        } else {
            quote!()
        };
        let module_name_tokens = quote_escaped_ident(name.to_snake_case());
        quote!(
            pub mod #module_name_tokens {
                use daml::prelude::*;
                #module_tokens
                #( #all_children )*
            }
        )
    }
}
