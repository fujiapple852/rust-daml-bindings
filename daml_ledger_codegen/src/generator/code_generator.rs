use proc_macro2::TokenStream;

use daml_lf::DarFile;

use crate::convert::archive::wrapper::DamlArchiveWrapper;
use crate::convert::archive::DamlArchivePayload;
use crate::convert::error::{DamlCodeGenError, DamlCodeGenResult};
use crate::element::DamlArchive;
use crate::generator::archive_code_generator::combined::generate_archive_combined;
use crate::generator::archive_code_generator::separate::generate_archive_separate;
use crate::generator::generator_options::RenderMethod;
use crate::generator::module_matcher::ModuleMatcher;
use crate::generator::ModuleOutputMode;
use crate::renderer::quote_archive;
use std::convert::TryFrom;

/// Code generator which is designed to be called from `build.rs` files.
///
/// TODO document this
pub fn daml_codegen(
    dar_file: &str,
    output_path: &str,
    module_filter_regex: &[&str],
    quote_method: RenderMethod,
    module_output_mode: ModuleOutputMode,
) -> DamlCodeGenResult<()> {
    println!("cargo:rerun-if-changed={}", dar_file);
    daml_codegen_internal(dar_file, output_path, module_filter_regex, quote_method, module_output_mode)
}

/// Generate a Rust `TokenStream` representing the supplied DAML Archive.
pub fn generate_tokens(
    dar_file: &DarFile,
    module_filter_regex: &[&str],
    render_method: &RenderMethod,
) -> DamlCodeGenResult<TokenStream> {
    let archive_payload = DamlArchivePayload::try_from(dar_file)?;
    let daml_archive = DamlArchiveWrapper::wrap(&archive_payload);
    let archive = DamlArchive::try_from(&daml_archive)?;
    let module_matcher = ModuleMatcher::new(module_filter_regex)?;
    Ok(quote_archive(&archive, &module_matcher, render_method))
}

#[doc(hidden)]
pub fn daml_codegen_internal(
    dar_file: &str,
    output_path: &str,
    module_filter_regex: &[&str],
    render_method: RenderMethod,
    module_output_mode: ModuleOutputMode,
) -> DamlCodeGenResult<()> {
    let dar = DarFile::from_file(dar_file).map_err(DamlCodeGenError::DamlLfError)?;
    let archive_payload = DamlArchivePayload::try_from(&dar)?;
    let daml_archive = DamlArchiveWrapper::wrap(&archive_payload);
    let archive = DamlArchive::try_from(&daml_archive)?;
    let module_matcher = ModuleMatcher::new(module_filter_regex)?;
    match module_output_mode {
        ModuleOutputMode::Separate =>
            generate_archive_separate(&archive, output_path.as_ref(), &module_matcher, &render_method)
                .map_err(DamlCodeGenError::IOError)?,
        ModuleOutputMode::Combined =>
            generate_archive_combined(&archive, output_path.as_ref(), &module_matcher, &render_method)
                .map_err(DamlCodeGenError::IOError)?,
    }
    Ok(())
}
