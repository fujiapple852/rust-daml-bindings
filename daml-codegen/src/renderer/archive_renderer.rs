use proc_macro2::TokenStream;
use quote::quote;

use crate::generator::{ModuleMatcher, RenderMethod};
use crate::renderer::render_context::RenderFilterMode;
use crate::renderer::{quote_package, RenderContext};
use daml_lf::element::DamlArchive;

pub fn quote_archive(
    archive: &DamlArchive<'_>,
    module_matcher: &ModuleMatcher,
    render_method: &RenderMethod,
) -> TokenStream {
    let ctx = RenderContext::with_archive(archive, RenderFilterMode::default());
    let all_packages: Vec<_> =
        archive.packages().map(|package| quote_package(&ctx, package, module_matcher, render_method)).collect();
    quote!(
        #(
            #[allow(clippy::all, warnings)]
            #all_packages
        )*
    )
}
