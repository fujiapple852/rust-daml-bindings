use daml::json_api::schema_encoder::{ReferenceMode, RenderTitle};

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

///
#[derive(Copy, Clone, Debug)]
pub enum PathStyle {
    Fragment,
    Slash,
}

impl PathStyle {
    pub const fn separator(self) -> char {
        match self {
            Self::Fragment => '#',
            Self::Slash => '/',
        }
    }
}

impl Default for PathStyle {
    fn default() -> Self {
        Self::Fragment
    }
}
