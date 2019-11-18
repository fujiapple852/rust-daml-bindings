use proc_macro2::TokenStream;

use crate::element::*;
use crate::generator::ModuleMatcher;
use crate::generator::RenderMethod;
use crate::renderer::quote_module_tree;

pub fn quote_package(
    package: &DamlPackage,
    module_matcher: &ModuleMatcher,
    render_method: &RenderMethod,
) -> TokenStream {
    quote_module_tree(package.name, &package.root_module, module_matcher, render_method)
}
