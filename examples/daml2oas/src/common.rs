//! Common items
use daml::json_api::request_converter::DamlJsonTemplateId;

/// The name given to the JSON schema item holding the Daml JSON error response.
pub const ERROR_RESPONSE_SCHEMA_NAME: &str = "ErrorResponse";

/// The name of the `Archive` choice available on all templates.
pub const ARCHIVE_CHOICE_NAME: &str = "Archive";

/// A fully qualified entity.
pub type DataId = DamlJsonTemplateId;

///
pub struct NamedItem<T> {
    pub name: String,
    pub item: T,
}

impl<T> NamedItem<T> {
    pub const fn new(name: String, item: T) -> Self {
        Self {
            name,
            item,
        }
    }
}
