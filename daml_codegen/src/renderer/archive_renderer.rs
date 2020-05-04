use proc_macro2::TokenStream;
use quote::quote;

use crate::generator::{ModuleMatcher, RenderMethod};
use crate::renderer::{quote_package, RenderContext};
use daml_lf::element::DamlArchive;

pub fn quote_archive(
    archive: &DamlArchive<'_>,
    module_matcher: &ModuleMatcher,
    render_method: &RenderMethod,
) -> TokenStream {
    let ctx = RenderContext::with_archive(archive);
    let all_packages: Vec<_> = archive
        .packages()
        .values()
        .map(|package| quote_package(&ctx, package, module_matcher, render_method))
        .collect();
    quote!(
        #(
            #[allow(clippy::all, warnings)]
            #all_packages
        )*
    )
}
