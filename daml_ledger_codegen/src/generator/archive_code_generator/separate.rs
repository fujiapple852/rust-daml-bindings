use crate::element::{DamlArchive, DamlModule, DamlPackage};
use crate::generator::module_matcher::ModuleMatcher;
use crate::generator::RenderMethod;
use crate::renderer::{quote_all_data, to_module_path, to_rust_identifier};
use itertools::Itertools;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::path::PathBuf;
use std::{fs, io};

const DISABLE_WARNINGS: &str = "#![allow(clippy::all, warnings)]";
const USE_DAML_PRELUDE: &str = "use daml::prelude::*;";

pub fn generate_archive_separate(
    archive: &DamlArchive<'_>,
    output_path: &Path,
    module_matcher: &ModuleMatcher,
    render_method: &RenderMethod,
) -> Result<(), Error> {
    for package in archive.packages.values() {
        generate_package_source(package, output_path, module_matcher, render_method)?
    }
    Ok(())
}

fn generate_package_source(
    package: &DamlPackage<'_>,
    output_path: &Path,
    module_matcher: &ModuleMatcher,
    render_method: &RenderMethod,
) -> Result<(), Error> {
    let root_modules: Vec<_> =
        package.root_module.child_modules.values().filter(|&m| is_interesting_module(m, module_matcher)).collect();
    let module_decl = root_modules.iter().map(|m| make_pub_mod_declaration(m.name())).join("\n");
    let package_body = format!("{}\n{}", DISABLE_WARNINGS, module_decl);
    let mut package_file =
        create_file(&PathBuf::from(output_path), &make_package_filename(package.name, package.version))?;
    package_file.write_all(package_body.as_bytes())?;
    let package_dir_path = PathBuf::from(output_path).join(&make_package_name(package.name, package.version));
    for module in root_modules {
        generate_module_source(module, &package_dir_path, module_matcher, render_method)?;
    }
    Ok(())
}

fn generate_module_source(
    module: &DamlModule<'_>,
    package_dir_path: &Path,
    module_matcher: &ModuleMatcher,
    render_method: &RenderMethod,
) -> Result<(), Error> {
    let sub_modules: Vec<_> =
        module.child_modules.values().filter(|&m| is_interesting_module(m, module_matcher)).collect();
    let sub_module_decl: String = sub_modules.iter().map(|&m| make_pub_mod_declaration(m.name())).join("\n");
    let module_types_text = quote_module_data_types(module, render_method);
    let module_body = format!("{}\n{}{}", USE_DAML_PRELUDE, sub_module_decl, module_types_text);
    let module_dir_path = module.path[..module.path.len() - 1].iter().map(to_rust_identifier).join("/");
    let package_module_dir_path = PathBuf::from(package_dir_path).join(module_dir_path);
    let mut module_file = create_file(&package_module_dir_path, &make_module_filename(module.name()))?;
    module_file.write_all(module_body.as_bytes())?;
    for child_module in sub_modules {
        generate_module_source(child_module, package_dir_path, module_matcher, render_method)?;
    }
    Ok(())
}

fn is_interesting_module(module: &DamlModule<'_>, module_matcher: &ModuleMatcher) -> bool {
    (!module.data_types.is_empty() && module_matcher.matches(&to_module_path(module.path.as_slice())))
        || module.child_modules.values().any(|m| is_interesting_module(m, module_matcher))
}

fn quote_module_data_types(module: &DamlModule<'_>, render_method: &RenderMethod) -> String {
    quote_all_data(module.data_types.values().collect::<Vec<_>>().as_slice(), render_method).to_string()
}

fn create_file(base_dir: &PathBuf, filename: &str) -> io::Result<File> {
    fs::create_dir_all(&base_dir)?;
    File::create(base_dir.join(filename))
}

fn make_package_filename(name: &str, version: Option<&str>) -> String {
    format!("{}.rs", make_package_name(name, version))
}

fn make_package_name(name: &str, version: Option<&str>) -> String {
    if let Some(v) = version {
        format!("{}_{}", to_rust_identifier(name), to_rust_identifier(v))
    } else {
        to_rust_identifier(name)
    }
}

fn make_module_filename(name: &str) -> String {
    format!("{}.rs", to_rust_identifier(name))
}

fn make_pub_mod_declaration(module_name: &str) -> String {
    format!("pub mod {};", to_rust_identifier(module_name))
}
