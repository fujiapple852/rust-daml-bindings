//! Formatting function.
use crate::common::DataId;
use daml::lf::element::{DamlData, DamlModule};
use itertools::Itertools;

const PATH_SEPARATOR: &str = ".";
const PATH_SEPARATOR_REGEX_SAFE: &str = "\\.";

/// Format a `TemplateId` that is suitable for use in Daml JSON API payloads.
pub fn format_daml_template(template_id: &DataId) -> String {
    format!("{}:{}", format_path(&template_id.module), template_id.entity)
}

/// Format a `TemplateId` that is suitable for JSON schema and OAS references.
///
/// The Daml JSON separator character ':' is not legal in URLs and so cannot be used in OAS paths and so we
/// use `.` instead.
pub fn format_oas_template(template_id: &DataId) -> String {
    format!("{}.{}", format_path(&template_id.module), template_id.entity)
}

/// Format an OAS path for a `DamlData` in a `DamlModule`.
///
/// DOCME
pub fn format_oas_data(module: &DamlModule<'_>, data: &DamlData<'_>) -> String {
    format!("{}.{}", format_path_join(module.path(), PATH_SEPARATOR), data.name())
}

/// Format a `TemplateId` choice that is suitable for JSON schema and OAS references.
///
/// Note that Daml choices are modelled as data types at the Module level and so we do no include the template name.
pub fn format_oas_template_choice(template_id: &DataId, choice: &str) -> String {
    format!("{}.{}", format_path(&template_id.module), choice)
}

/// Format a module path.
pub fn format_path<T: AsRef<str>>(path: &[T]) -> String {
    format_path_join(path.iter().map(AsRef::as_ref), PATH_SEPARATOR)
}

/// Format a url safe module path.
pub fn format_path_regex_safe<T: AsRef<str>>(path: &[T]) -> String {
    format_path_join(path.iter().map(AsRef::as_ref), PATH_SEPARATOR_REGEX_SAFE)
}

fn format_path_join<'a>(mut path: impl Iterator<Item = &'a str>, sep: &str) -> String {
    path.join(sep)
}
