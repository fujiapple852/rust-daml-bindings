use daml_lf::protobuf_autogen::daml_lf_1::*;
use daml_lf::{LanguageFeatureVersion, LanguageVersion};
use std::fmt;
use syn::export::fmt::Error;
use syn::export::Formatter;

/// Resolve DAML package interned dotted names and strings.
pub trait PackageInternedResolver {
    fn package_id(&self) -> &str;
    fn language_version(&self) -> LanguageVersion;
    fn interned_strings(&self) -> &[String];
    fn interned_dotted_names(&self) -> &[&[i32]];

    /// Resolve an interned string by index.
    fn resolve_string(&self, index: i32) -> &str {
        self.interned_strings().get(index as usize).map(AsRef::as_ref).expect("InternableString")
    }

    /// Resolve multiple interned strings by indices.
    fn resolve_strings(&self, indices: &[i32]) -> Vec<&str> {
        indices.iter().map(|&i| self.resolve_string(i)).collect()
    }

    /// Partially resolve an interned dotted name by index to interned string indices.
    fn resolve_dotted_to_indices(&self, index: i32) -> &[i32] {
        self.interned_dotted_names().get(index as usize).expect("InternableDottedName::Interned")
    }

    /// Fully resolve an interned dotted name by index.
    fn resolve_dotted(&self, index: i32) -> Vec<&str> {
        self.resolve_strings(self.resolve_dotted_to_indices(index))
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum InternableString<'a> {
    Literal(&'a str),
    Interned(i32),
}

impl<'a> fmt::Display for InternableString<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            InternableString::Literal(s) => write!(f, "InternableString::Literal({})", s),
            InternableString::Interned(i) => write!(f, "InternableString::Interned({})", i),
        }
    }
}

impl<'a> InternableString<'a> {
    pub fn resolve(&self, resolver: &'a impl PackageInternedResolver) -> &'a str {
        match *self {
            InternableString::Literal(s) => {
                assert_does_not_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_STRINGS);
                s
            },
            InternableString::Interned(i) => {
                assert_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_STRINGS);
                resolver.resolve_string(i)
            },
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum InternableDottedName<'a> {
    Literal(&'a [String]),
    Interned(i32),
}

impl<'a> InternableDottedName<'a> {
    pub fn resolve(&self, resolver: &'a impl PackageInternedResolver) -> Vec<&'a str> {
        match *self {
            InternableDottedName::Literal(dn) => {
                assert_does_not_support_feature(
                    resolver.language_version(),
                    &LanguageFeatureVersion::INTERNED_DOTTED_NAMES,
                );
                dn.iter().map(AsRef::as_ref).collect()
            },
            InternableDottedName::Interned(i) => {
                assert_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_DOTTED_NAMES);
                resolver.resolve_dotted(i)
            },
        }
    }

    pub fn resolve_last(&self, resolver: &'a impl PackageInternedResolver) -> &'a str {
        match *self {
            InternableDottedName::Literal(dn) => {
                assert_does_not_support_feature(
                    resolver.language_version(),
                    &LanguageFeatureVersion::INTERNED_DOTTED_NAMES,
                );
                dn.last().map(String::as_str).expect("InternableDottedName::Literal")
            },
            InternableDottedName::Interned(i) => {
                assert_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_DOTTED_NAMES);
                resolver
                    .resolve_dotted_to_indices(i)
                    .last()
                    .map(|&j| resolver.resolve_string(j))
                    .expect("InternableDottedName::Interned")
            },
        }
    }
}

fn assert_support_feature(current_version: LanguageVersion, feature_version: &LanguageFeatureVersion) {
    if !current_version.supports_feature(feature_version) {
        panic!(
            "DAML LF version {} does not support feature {} (requires version {})",
            current_version, feature_version.name, feature_version.min_version
        );
    }
}

fn assert_does_not_support_feature(current_version: LanguageVersion, feature_version: &LanguageFeatureVersion) {
    if current_version.supports_feature(feature_version) {
        panic!(
            "DAML LF version {} supports feature {} but was not used (supported as of version {})",
            current_version, feature_version.name, feature_version.min_version
        );
    }
}

impl<'a> fmt::Display for InternableDottedName<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            InternableDottedName::Literal(dn) => write!(f, "InternableDottedName::Literal({})", dn.join(".")),
            InternableDottedName::Interned(i) => write!(f, "InternableDottedName::Interned({})", i),
        }
    }
}

impl<'a> From<&'a field_with_type::Field> for InternableString<'a> {
    fn from(field: &'a field_with_type::Field) -> Self {
        match field {
            field_with_type::Field::FieldStr(s) => InternableString::Literal(s.as_str()),
            &field_with_type::Field::FieldInternedStr(i) => InternableString::Interned(i),
        }
    }
}

impl<'a> From<&'a template_choice::Name> for InternableString<'a> {
    fn from(name: &'a template_choice::Name) -> Self {
        match name {
            template_choice::Name::NameStr(s) => InternableString::Literal(s.as_str()),
            &template_choice::Name::NameInternedStr(i) => InternableString::Interned(i),
        }
    }
}

impl<'a> From<&'a def_data_type::Name> for InternableDottedName<'a> {
    fn from(name: &'a def_data_type::Name) -> Self {
        match name {
            def_data_type::Name::NameDname(dn) => InternableDottedName::Literal(dn.segments.as_slice()),
            &def_data_type::Name::NameInternedDname(i) => InternableDottedName::Interned(i),
        }
    }
}

impl<'a> From<&'a def_template::Tycon> for InternableDottedName<'a> {
    fn from(ty_con: &'a def_template::Tycon) -> Self {
        match ty_con {
            def_template::Tycon::TyconDname(dn) => InternableDottedName::Literal(dn.segments.as_slice()),
            &def_template::Tycon::TyconInternedDname(i) => InternableDottedName::Interned(i),
        }
    }
}

impl<'a> From<&'a module::Name> for InternableDottedName<'a> {
    fn from(name: &'a module::Name) -> Self {
        match name {
            module::Name::NameDname(dn) => InternableDottedName::Literal(dn.segments.as_slice()),
            &module::Name::NameInternedDname(i) => InternableDottedName::Interned(i),
        }
    }
}

impl<'a> From<&'a module_ref::ModuleName> for InternableDottedName<'a> {
    fn from(name: &'a module_ref::ModuleName) -> Self {
        match name {
            module_ref::ModuleName::ModuleNameDname(dn) => InternableDottedName::Literal(dn.segments.as_slice()),
            &module_ref::ModuleName::ModuleNameInternedDname(i) => InternableDottedName::Interned(i),
        }
    }
}

impl<'a> From<&'a type_con_name::Name> for InternableDottedName<'a> {
    fn from(name: &'a type_con_name::Name) -> Self {
        match name {
            type_con_name::Name::NameDname(dn) => InternableDottedName::Literal(dn.segments.as_slice()),
            &type_con_name::Name::NameInternedDname(i) => InternableDottedName::Interned(i),
        }
    }
}
