use crate::convert::util::Required;
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::lf_protobuf::com::digitalasset::daml_lf_1::{
    def_data_type, def_template, field_with_type, module, module_ref, r#type, template_choice, type_con_name,
    type_var_with_kind,
};
use crate::{LanguageFeatureVersion, LanguageVersion};
use std::fmt;
use std::fmt::{Error, Formatter};

/// Resolve DAML package interned dotted names and strings.
pub trait PackageInternedResolver {
    fn package_id(&self) -> &str;
    fn language_version(&self) -> LanguageVersion;
    fn interned_strings(&self) -> &[String];
    fn interned_dotted_names(&self) -> &[&[i32]];

    /// Resolve an interned string by index.
    fn resolve_string(&self, index: i32) -> DamlLfConvertResult<&str> {
        Ok(self.interned_strings().get(index as usize).map(AsRef::as_ref).req()?)
    }

    /// Resolve multiple interned strings by indices.
    fn resolve_strings(&self, indices: &[i32]) -> DamlLfConvertResult<Vec<&str>> {
        Ok(indices.iter().map(|&i| self.resolve_string(i)).collect::<DamlLfConvertResult<_>>()?)
    }

    /// Partially resolve an interned dotted name by index to interned string indices.
    fn resolve_dotted_to_indices(&self, index: i32) -> DamlLfConvertResult<&[i32]> {
        Ok(self.interned_dotted_names().get(index as usize).req()?)
    }

    /// Fully resolve an interned dotted name by index.
    fn resolve_dotted(&self, index: i32) -> DamlLfConvertResult<Vec<&str>> {
        Ok(self.resolve_strings(self.resolve_dotted_to_indices(index)?)?)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum InternableString<'a> {
    LiteralString(&'a str),
    InternedString(i32),
}

impl<'a> fmt::Display for InternableString<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            InternableString::LiteralString(s) => write!(f, "InternableString::Literal({})", s),
            InternableString::InternedString(i) => write!(f, "InternableString::Interned({})", i),
        }
    }
}

impl<'a> InternableString<'a> {
    pub fn resolve(&self, resolver: &'a impl PackageInternedResolver) -> DamlLfConvertResult<&'a str> {
        Ok(match *self {
            InternableString::LiteralString(s) => {
                check_does_not_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_STRINGS)?;
                s
            },
            InternableString::InternedString(i) => {
                check_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_STRINGS)?;
                resolver.resolve_string(i)?
            },
        })
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum InternableDottedName<'a> {
    LiteralDottedName(&'a [String]),
    InternedDottedName(i32),
}

impl<'a> InternableDottedName<'a> {
    pub fn resolve(&self, resolver: &'a impl PackageInternedResolver) -> DamlLfConvertResult<Vec<&'a str>> {
        Ok(match *self {
            InternableDottedName::LiteralDottedName(dn) => {
                check_does_not_support_feature(
                    resolver.language_version(),
                    &LanguageFeatureVersion::INTERNED_DOTTED_NAMES,
                )?;
                dn.iter().map(AsRef::as_ref).collect()
            },
            InternableDottedName::InternedDottedName(i) => {
                check_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_DOTTED_NAMES)?;
                resolver.resolve_dotted(i)?
            },
        })
    }

    pub fn resolve_last(&self, resolver: &'a impl PackageInternedResolver) -> DamlLfConvertResult<&'a str> {
        Ok(match *self {
            InternableDottedName::LiteralDottedName(dn) => {
                check_does_not_support_feature(
                    resolver.language_version(),
                    &LanguageFeatureVersion::INTERNED_DOTTED_NAMES,
                )?;
                dn.last().map(String::as_str).req()?
            },
            InternableDottedName::InternedDottedName(i) => {
                check_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_DOTTED_NAMES)?;
                resolver.resolve_dotted_to_indices(i)?.last().req().and_then(|&j| resolver.resolve_string(j))?
            },
        })
    }
}

fn check_support_feature(
    current_version: LanguageVersion,
    feature_version: &LanguageFeatureVersion,
) -> DamlLfConvertResult<()> {
    if current_version.supports_feature(feature_version) {
        Ok(())
    } else {
        Err(DamlLfConvertError::UnsupportedFeatureUsed(
            current_version.to_string(),
            feature_version.name.to_owned(),
            feature_version.min_version.to_string(),
        ))
    }
}

fn check_does_not_support_feature(
    current_version: LanguageVersion,
    feature_version: &LanguageFeatureVersion,
) -> DamlLfConvertResult<()> {
    if current_version.supports_feature(feature_version) {
        Err(DamlLfConvertError::SupportedFeatureUnused(
            current_version.to_string(),
            feature_version.name.to_owned(),
            feature_version.min_version.to_string(),
        ))
    } else {
        Ok(())
    }
}

impl<'a> fmt::Display for InternableDottedName<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            InternableDottedName::LiteralDottedName(dn) => write!(f, "InternableDottedName::Literal({})", dn.join(".")),
            InternableDottedName::InternedDottedName(i) => write!(f, "InternableDottedName::Interned({})", i),
        }
    }
}

impl<'a> From<&'a field_with_type::Field> for InternableString<'a> {
    fn from(field: &'a field_with_type::Field) -> Self {
        match field {
            field_with_type::Field::FieldStr(s) => InternableString::LiteralString(s.as_str()),
            &field_with_type::Field::FieldInternedStr(i) => InternableString::InternedString(i),
        }
    }
}

impl<'a> From<&'a template_choice::Name> for InternableString<'a> {
    fn from(name: &'a template_choice::Name) -> Self {
        match name {
            template_choice::Name::NameStr(s) => InternableString::LiteralString(s.as_str()),
            &template_choice::Name::NameInternedStr(i) => InternableString::InternedString(i),
        }
    }
}

impl<'a> From<&'a type_var_with_kind::Var> for InternableString<'a> {
    fn from(name: &'a type_var_with_kind::Var) -> Self {
        match name {
            type_var_with_kind::Var::VarStr(s) => InternableString::LiteralString(s.as_str()),
            &type_var_with_kind::Var::VarInternedStr(i) => InternableString::InternedString(i),
        }
    }
}

impl<'a> From<&'a r#type::var::Var> for InternableString<'a> {
    fn from(name: &'a r#type::var::Var) -> Self {
        match name {
            r#type::var::Var::VarStr(s) => InternableString::LiteralString(s.as_str()),
            &r#type::var::Var::VarInternedStr(i) => InternableString::InternedString(i),
        }
    }
}

impl<'a> From<&'a def_data_type::Name> for InternableDottedName<'a> {
    fn from(name: &'a def_data_type::Name) -> Self {
        match name {
            def_data_type::Name::NameDname(dn) => InternableDottedName::LiteralDottedName(dn.segments.as_slice()),
            &def_data_type::Name::NameInternedDname(i) => InternableDottedName::InternedDottedName(i),
        }
    }
}

impl<'a> From<&'a def_template::Tycon> for InternableDottedName<'a> {
    fn from(ty_con: &'a def_template::Tycon) -> Self {
        match ty_con {
            def_template::Tycon::TyconDname(dn) => InternableDottedName::LiteralDottedName(dn.segments.as_slice()),
            &def_template::Tycon::TyconInternedDname(i) => InternableDottedName::InternedDottedName(i),
        }
    }
}

impl<'a> From<&'a module::Name> for InternableDottedName<'a> {
    fn from(name: &'a module::Name) -> Self {
        match name {
            module::Name::NameDname(dn) => InternableDottedName::LiteralDottedName(dn.segments.as_slice()),
            &module::Name::NameInternedDname(i) => InternableDottedName::InternedDottedName(i),
        }
    }
}

impl<'a> From<&'a module_ref::ModuleName> for InternableDottedName<'a> {
    fn from(name: &'a module_ref::ModuleName) -> Self {
        match name {
            module_ref::ModuleName::ModuleNameDname(dn) =>
                InternableDottedName::LiteralDottedName(dn.segments.as_slice()),
            &module_ref::ModuleName::ModuleNameInternedDname(i) => InternableDottedName::InternedDottedName(i),
        }
    }
}

impl<'a> From<&'a type_con_name::Name> for InternableDottedName<'a> {
    fn from(name: &'a type_con_name::Name) -> Self {
        match name {
            type_con_name::Name::NameDname(dn) => InternableDottedName::LiteralDottedName(dn.segments.as_slice()),
            &type_con_name::Name::NameInternedDname(i) => InternableDottedName::InternedDottedName(i),
        }
    }
}
