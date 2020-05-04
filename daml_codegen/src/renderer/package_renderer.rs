use proc_macro2::TokenStream;

use crate::generator::ModuleMatcher;
use crate::generator::RenderMethod;
use crate::renderer::{quote_module_tree, RenderContext};
use daml_lf::element::DamlPackage;

pub fn quote_package(
    ctx: &RenderContext<'_>,
    package: &DamlPackage<'_>,
    module_matcher: &ModuleMatcher,
    render_method: &RenderMethod,
) -> TokenStream {
    quote_module_tree(ctx, package.name(), package.root_module(), module_matcher, render_method)
}
