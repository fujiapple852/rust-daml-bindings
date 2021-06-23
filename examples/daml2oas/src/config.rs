use daml::json_api::schema_encoder::{ReferenceMode, RenderTitle};

use crate::operation::PathStyle;

#[derive(Copy, Clone, Debug)]
pub enum OutputFormat {
    Json,
    Yaml,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Json
    }
}

///
pub struct Config<'a> {
    pub dar_file: String,
    pub companion_file: String,
    pub format: OutputFormat,
    pub output_file: Option<String>,
    pub module_path: Vec<&'a str>,
    pub render_title: RenderTitle,
    pub reference_prefix: &'a str,
    pub reference_mode: ReferenceMode,
    pub emit_package_id: bool,
    pub include_archive_choice: bool,
    pub path_style: PathStyle,
}
