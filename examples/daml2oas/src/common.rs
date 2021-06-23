//! Common items

/// The name given to the JSON schema item holding the Daml JSON error response.
pub const ERROR_RESPONSE_SCHEMA_NAME: &str = "ErrorResponse";

/// The name of the `Archive` choice available on all templates.
pub const ARCHIVE_CHOICE_NAME: &str = "Archive";

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

/// A Daml data Id.
#[derive(Debug, Clone)]
pub struct DataId {
    pub package_id: String,
    pub module_path: Vec<String>,
    pub name: String,
}

impl DataId {
    pub const fn new(package_id: String, module_path: Vec<String>, name: String) -> Self {
        Self {
            package_id,
            module_path,
            name,
        }
    }
}
