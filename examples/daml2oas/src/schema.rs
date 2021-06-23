use serde::Serialize;
use serde_json::Value;

/// Represents a JSON Schema object.
#[derive(Debug, Serialize)]
pub struct Schema {
    #[serde(flatten)]
    pub value: Value,
}

impl Schema {
    pub const fn new(value: Value) -> Self {
        Self {
            value,
        }
    }
}
