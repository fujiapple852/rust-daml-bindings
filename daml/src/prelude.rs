#![allow(unused_imports, unused)]
pub use daml_ledger_api::data::command::{DamlCommand, DamlCreateCommand, DamlExerciseCommand};
pub use daml_ledger_api::data::event::DamlCreatedEvent;
pub use daml_ledger_api::data::value::{DamlEnum, DamlRecord, DamlRecordField, DamlValue, DamlVariant};
pub use daml_ledger_api::data::DamlError;
pub use daml_ledger_api::data::DamlIdentifier;
pub use daml_ledger_api::data::DamlResult;
pub use daml_ledger_api::{CommandExecutor, Executor};
pub use daml_ledger_derive::DamlChoices;
pub use daml_ledger_derive::DamlData;
pub use daml_ledger_derive::DamlEnum;
pub use daml_ledger_derive::DamlTemplate;
pub use daml_ledger_derive::DamlVariant;

/// Type alias for a DAML `ContractId`.
pub type DamlContractId = String;

/// Type alias for a DAML `Int`.
pub type DamlInt64 = i64;

/// Type alias for a DAML `Numeric`.
pub type DamlNumeric = daml_ledger_api::bigdecimal::BigDecimal;

/// Type alias for a DAML `Text`.
pub type DamlText = String;

/// Type alias for a DAML `Timestamp`.
pub type DamlTimestamp = daml_ledger_api::chrono::DateTime<daml_ledger_api::chrono::Utc>;

/// Type alias for a DAML `Party`.
pub type DamlParty = String;

/// Type alias for a DAML `Bool`.
pub type DamlBool = bool;

/// Type alias for a DAML `Unit`.
pub type DamlUnit = ();

/// Type alias for a DAML `Date`.
pub type DamlDate = daml_ledger_api::chrono::Date<daml_ledger_api::chrono::Utc>;

/// Type alias for a DAML `List a`.
pub type DamlList<T> = Vec<T>;

/// Type alias for a DAML `TextMap a`.
pub type DamlTextMap<T> = HashMap<String, T>;

/// Type alias for a DAML `Optional a`.
pub type DamlOptional<T> = Option<T>;

#[doc(hidden)]
pub use std::collections::HashMap;
#[doc(hidden)]
pub use std::convert::{TryFrom, TryInto};
