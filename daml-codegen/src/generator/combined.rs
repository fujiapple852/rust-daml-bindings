use crate::generator::module_matcher::ModuleMatcher;
use crate::generator::RenderMethod;
use crate::renderer::{quote_archive, to_rust_identifier};
use daml_lf::element::DamlArchive;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::{fs, io};

pub fn generate_archive_combined(
    archive: &DamlArchive<'_>,
    output_path: &Path,
    module_matcher: &ModuleMatcher,
    render_method: &RenderMethod,
) -> Result<(), io::Error> {
    let rendered = quote_archive(archive, module_matcher, render_method).to_string();
    fs::create_dir_all(output_path)?;
    let generated_filename = format!("{}.rs", to_rust_identifier(archive.name()));
    let target = PathBuf::from(output_path).join(generated_filename);
    let mut output_file = File::create(target)?;
    output_file.write_all(rendered.as_bytes())?;
    Ok(())
}
