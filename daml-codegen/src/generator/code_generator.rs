use daml_lf::DarFile;

use crate::error::{DamlCodeGenError, DamlCodeGenResult};
use crate::generator::combined::generate_archive_combined;
use crate::generator::generator_options::RenderMethod;
use crate::generator::module_matcher::ModuleMatcher;
use crate::generator::separate::generate_archive_separate;
use crate::generator::ModuleOutputMode;

/// Code generator which is designed to be called from `build.rs` files.
///
/// To use the [`daml_codegen`] function you must first import the `daml` crate and specify feature `codegen`:
///
/// ```toml
/// [dependencies]
/// daml = { version = "0.1.1", features = [ "codegen" ] }
/// ```
///
/// In your `build.rs` main function invoke the [`daml_codegen`] function for a `dar` file and specify where the
/// generated src code should be created:
///
/// ```rust
/// use daml_codegen::generator::{daml_codegen, ModuleOutputMode, RenderMethod};
/// fn main() {
///     let method = RenderMethod::Full;
///     let mode = ModuleOutputMode::Combined;
///     daml_codegen("MyModel.dar", "src/autogen", &[], method, mode).unwrap();
/// }
/// ```
///
/// In the example above we used [`RenderMethod::Full`] to indicate that we want to render Rust types without
/// intermediate annotations (such as [`DamlTemplate`]) and [`ModuleOutputMode::Combined`] to combine all Daml modules
/// in a single Rust src file.
///
/// [`DamlTemplate`]: https://docs.rs/daml-derive/0.1.1/daml_derive/attr.DamlTemplate.html
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

#[doc(hidden)]
pub fn daml_codegen_internal(
    dar_file: &str,
    output_path: &str,
    module_filter_regex: &[&str],
    render_method: RenderMethod,
    module_output_mode: ModuleOutputMode,
) -> DamlCodeGenResult<()> {
    let dar = DarFile::from_file(dar_file).map_err(DamlCodeGenError::DamlLfError)?;
    dar.apply(|archive| {
        let module_matcher = ModuleMatcher::new(module_filter_regex)?;
        match module_output_mode {
            ModuleOutputMode::Separate =>
                generate_archive_separate(archive, output_path.as_ref(), &module_matcher, &render_method)
                    .map_err(DamlCodeGenError::IoError)?,
            ModuleOutputMode::Combined =>
                generate_archive_combined(archive, output_path.as_ref(), &module_matcher, &render_method)
                    .map_err(DamlCodeGenError::IoError)?,
        }
        Ok(())
    })
    .map_err(DamlCodeGenError::DamlLfError)?
}
