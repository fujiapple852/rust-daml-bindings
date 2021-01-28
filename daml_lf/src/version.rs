use crate::DamlLfError;
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt::{Display, Error, Formatter};

/// DAML Ledger Fragment language version.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub enum LanguageVersion {
    LV0,
    LV1(LanguageV1MinorVersion),
}

impl LanguageVersion {
    pub const V0: LanguageVersion = LanguageVersion::LV0;
    pub const V1_0: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::V0);
    pub const V1_1: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::V1);
    pub const V1_11: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::V11);
    pub const V1_2: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::V2);
    pub const V1_3: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::V3);
    pub const V1_4: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::V4);
    pub const V1_5: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::V5);
    pub const V1_6: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::V6);
    pub const V1_7: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::V7);
    pub const V1_8: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::V8);
    pub const V1_DEV: LanguageVersion = LanguageVersion::LV1(LanguageV1MinorVersion::Dev);

    pub fn new_v1(minor: LanguageV1MinorVersion) -> Self {
        LanguageVersion::LV1(minor)
    }

    pub fn supports_feature(self, feature_version: &LanguageFeatureVersion) -> bool {
        self >= feature_version.min_version
    }
}

impl Display for LanguageVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            LanguageVersion::LV0 => write!(f, "v0"),
            LanguageVersion::LV1(minor) => write!(f, "v1.{}", minor),
        }
    }
}

/// Minimum DAML LF language version support for a feature.
pub struct LanguageFeatureVersion {
    pub name: &'static str,
    pub min_version: LanguageVersion,
}

impl LanguageFeatureVersion {
    pub const ANY_TYPE: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "ANY_TYPE",
        min_version: LanguageVersion::V1_7,
    };
    pub const ARROW_TYPE: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "ARROW_TYPE",
        min_version: LanguageVersion::V1_1,
    };
    pub const CHOICE_OBSERVERS: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "CHOICE_OBSERVERS",
        min_version: LanguageVersion::V1_11,
    };
    pub const COERCE_CONTRACT_ID: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "COERCE_CONTRACT_ID",
        min_version: LanguageVersion::V1_5,
    };
    pub const COMPLEX_CONTACT_KEYS: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "COMPLEX_CONTACT_KEYS",
        min_version: LanguageVersion::V1_4,
    };
    pub const CONTRACT_KEYS: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "CONTRACT_KEYS",
        min_version: LanguageVersion::V1_3,
    };
    pub const DEFAULT: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "DEFAULT",
        min_version: LanguageVersion::V1_0,
    };
    pub const ENUM: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "ENUM",
        min_version: LanguageVersion::V1_6,
    };
    pub const INTERNED_DOTTED_NAMES: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "INTERNED_DOTTED_NAMES",
        min_version: LanguageVersion::V1_7,
    };
    pub const INTERNED_PACKAGE_ID: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "INTERNED_PACKAGE_ID",
        min_version: LanguageVersion::V1_6,
    };
    pub const INTERNED_STRINGS: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "INTERNED_STRINGS",
        min_version: LanguageVersion::V1_7,
    };
    pub const NUMBER_PARSING: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "NUMBER_PARSING",
        min_version: LanguageVersion::V1_5,
    };
    pub const NUMERIC: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "NUMERIC",
        min_version: LanguageVersion::V1_7,
    };
    pub const OPTIONAL: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "OPTIONAL",
        min_version: LanguageVersion::V1_1,
    };
    pub const OPTIONAL_EXERCISE_ACTOR: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "OPTIONAL_EXERCISE_ACTOR",
        min_version: LanguageVersion::V1_5,
    };
    pub const PACKAGE_METADATA: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "PACKAGE_METADATA",
        min_version: LanguageVersion::V1_8,
    };
    pub const PARTY_ORDERING: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "PARTY_ORDERING",
        min_version: LanguageVersion::V1_1,
    };
    pub const PARTY_TEXT_CONVERSIONS: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "PARTY_TEXT_CONVERSIONS",
        min_version: LanguageVersion::V1_2,
    };
    pub const SHA_TEXT: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "SHA_TEXT",
        min_version: LanguageVersion::V1_2,
    };
    pub const TEXTMAP: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "TEXTMAP",
        min_version: LanguageVersion::V1_3,
    };
    pub const TEXT_PACKING: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "TEXT_PACKING",
        min_version: LanguageVersion::V1_6,
    };
    pub const TYPE_REP: LanguageFeatureVersion = LanguageFeatureVersion {
        name: "TYPE_REP",
        min_version: LanguageVersion::V1_7,
    };
}

/// DAML Ledger Fragment language V1 minor version.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub enum LanguageV1MinorVersion {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V11,
    Dev,
}

impl Display for LanguageV1MinorVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            LanguageV1MinorVersion::V0 => write!(f, "0"),
            LanguageV1MinorVersion::V1 => write!(f, "1"),
            LanguageV1MinorVersion::V2 => write!(f, "2"),
            LanguageV1MinorVersion::V3 => write!(f, "3"),
            LanguageV1MinorVersion::V4 => write!(f, "4"),
            LanguageV1MinorVersion::V5 => write!(f, "5"),
            LanguageV1MinorVersion::V6 => write!(f, "6"),
            LanguageV1MinorVersion::V7 => write!(f, "7"),
            LanguageV1MinorVersion::V8 => write!(f, "8"),
            LanguageV1MinorVersion::V11 => write!(f, "11"),
            LanguageV1MinorVersion::Dev => write!(f, "dev"),
        }
    }
}

impl TryFrom<&str> for LanguageV1MinorVersion {
    type Error = DamlLfError;

    fn try_from(minor_version: &str) -> Result<Self, Self::Error> {
        match minor_version {
            "0" => Ok(LanguageV1MinorVersion::V0),
            "1" => Ok(LanguageV1MinorVersion::V1),
            "2" => Ok(LanguageV1MinorVersion::V2),
            "3" => Ok(LanguageV1MinorVersion::V3),
            "4" => Ok(LanguageV1MinorVersion::V4),
            "5" => Ok(LanguageV1MinorVersion::V5),
            "6" => Ok(LanguageV1MinorVersion::V6),
            "7" => Ok(LanguageV1MinorVersion::V7),
            "8" => Ok(LanguageV1MinorVersion::V8),
            "11" => Ok(LanguageV1MinorVersion::V11),
            "dev" => Ok(LanguageV1MinorVersion::Dev),
            _ => Err(DamlLfError::new_unknown_version(minor_version)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LanguageV1MinorVersion, LanguageVersion};

    #[test]
    fn test_minor_version_ordering() {
        assert!(LanguageV1MinorVersion::V0 < LanguageV1MinorVersion::V1);
        assert!(LanguageV1MinorVersion::V1 < LanguageV1MinorVersion::V2);
        assert!(LanguageV1MinorVersion::V2 < LanguageV1MinorVersion::V3);
        assert!(LanguageV1MinorVersion::V3 < LanguageV1MinorVersion::V4);
        assert!(LanguageV1MinorVersion::V4 < LanguageV1MinorVersion::V5);
        assert!(LanguageV1MinorVersion::V5 < LanguageV1MinorVersion::V6);
        assert!(LanguageV1MinorVersion::V6 < LanguageV1MinorVersion::V7);
        assert!(LanguageV1MinorVersion::V7 < LanguageV1MinorVersion::V8);
        assert!(LanguageV1MinorVersion::V8 < LanguageV1MinorVersion::V11);
        assert!(LanguageV1MinorVersion::V11 < LanguageV1MinorVersion::Dev);
    }

    #[test]
    fn test_version_matches() {
        assert_eq!(LanguageVersion::V1_7, LanguageVersion::LV1(LanguageV1MinorVersion::V7));
        assert_ne!(LanguageVersion::V1_7, LanguageVersion::V1_6);
    }

    #[test]
    fn test_version_ordering() {
        assert!(LanguageVersion::V0 < LanguageVersion::V1_0);
        assert!(LanguageVersion::V1_0 < LanguageVersion::V1_1);
        assert!(LanguageVersion::V1_1 < LanguageVersion::V1_2);
        assert!(LanguageVersion::V1_2 < LanguageVersion::V1_3);
        assert!(LanguageVersion::V1_3 < LanguageVersion::V1_4);
        assert!(LanguageVersion::V1_4 < LanguageVersion::V1_5);
        assert!(LanguageVersion::V1_5 < LanguageVersion::V1_6);
        assert!(LanguageVersion::V1_6 < LanguageVersion::V1_7);
        assert!(LanguageVersion::V1_7 < LanguageVersion::V1_8);
        assert!(LanguageVersion::V1_8 < LanguageVersion::V1_11);
        assert!(LanguageVersion::V1_11 < LanguageVersion::V1_DEV);
    }

    #[test]
    fn test_display_version() {
        assert_eq!("v0", LanguageVersion::V0.to_string());
        assert_eq!("v1.7", LanguageVersion::V1_7.to_string());
        assert_eq!("v1.dev", LanguageVersion::V1_DEV.to_string());
    }
}
