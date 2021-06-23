use crate::common::DataId;
use crate::config::PathStyle;
use crate::format::format_oas_template;

const API_VERSION_PATH: &str = "/v1";
const API_CREATE_PATH: &str = "create";
const API_CREATE_AND_EXERCISE_PATH: &str = "create_and_exercise";
const API_EXERCISE_PATH: &str = "exercise";
const API_FETCH_PATH: &str = "fetch";

/// A Daml OAS operation id maker.
#[derive(Debug, Clone, Copy)]
pub struct OperationIdFactory {
    path_style: PathStyle,
}

impl OperationIdFactory {
    pub const fn new(path_style: PathStyle) -> Self {
        Self {
            path_style,
        }
    }

    /// Format a `create` operation id.
    ///
    /// `/v1/create#Foo.Bar.MyTemplate`
    pub fn create_by_id(self, template_id: &DataId) -> String {
        format!(
            "{}/{}{}{}",
            API_VERSION_PATH,
            API_CREATE_PATH,
            self.path_style.separator(),
            format_oas_template(template_id)
        )
    }

    /// Format a `create_and_exercise` operation id.
    ///
    /// `/v1/create_and_exercise#Foo.Bar.MyTemplate/MyChoice`
    /// `/v1/create_and_exercise/Foo.Bar.MyTemplate/MyChoice`
    ///
    /// Note that choice names are unique within a module and so strictly speaking we do not need to include the
    /// Template name here however we do so for clarity.
    pub fn create_and_exercise(self, template_id: &DataId, choice: &str) -> String {
        format!(
            "{}/{}{}{}/{}",
            API_VERSION_PATH,
            API_CREATE_AND_EXERCISE_PATH,
            self.path_style.separator(),
            format_oas_template(template_id),
            choice
        )
    }

    /// Format a `exercise` operation id.
    ///
    /// `/v1/exercise#Foo.Bar.MyTemplate/MyChoice`
    /// `/v1/exercise/Foo.Bar.MyTemplate/MyChoice`
    ///
    /// Note that choice names are unique within a module and so strictly speaking we do not need to include the
    /// Template name here however we do so for clarity.
    pub fn exercise_by_id(self, template_id: &DataId, choice: &str) -> String {
        format!(
            "{}/{}{}{}/{}",
            API_VERSION_PATH,
            API_EXERCISE_PATH,
            self.path_style.separator(),
            format_oas_template(template_id),
            choice
        )
    }

    /// Format a `fetch` operation id.
    ///
    /// `/v1/fetch#Foo.Bar.MyTemplate`
    /// `/v1/fetch/Foo.Bar.MyTemplate`
    pub fn fetch_by_id(self, template_id: &DataId) -> String {
        format!(
            "{}/{}{}{}",
            API_VERSION_PATH,
            API_FETCH_PATH,
            self.path_style.separator(),
            format_oas_template(template_id),
        )
    }

    /// Format a `fetch` (by key) operation id.
    ///
    /// `/v1/fetch#key/Foo.Bar.MyTemplate`
    /// `/v1/fetch/key/Foo.Bar.MyTemplate`
    pub fn fetch_by_key(self, template_id: &DataId) -> String {
        format!(
            "{}/{}{}key/{}",
            API_VERSION_PATH,
            API_FETCH_PATH,
            self.path_style.separator(),
            format_oas_template(template_id),
        )
    }

    /// Format a `exercise` (by key) operation id.
    ///
    /// `/v1/exercise#key/Foo.Bar.MyTemplate/MyChoice`
    /// `/v1/exercise/key/Foo.Bar.MyTemplate/MyChoice`
    ///
    /// Note that choice names are unique within a module and so strictly speaking we do not need to include the
    /// Template name here however we do so for clarity.
    pub fn exercise_by_key(self, template_id: &DataId, choice: &str) -> String {
        format!(
            "{}/{}{}key/{}/{}",
            API_VERSION_PATH,
            API_EXERCISE_PATH,
            self.path_style.separator(),
            format_oas_template(template_id),
            choice
        )
    }
}
