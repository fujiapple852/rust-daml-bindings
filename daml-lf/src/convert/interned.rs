use crate::convert::type_payload::DamlTypePayload;
use crate::convert::util::Required;
use crate::error::{DamlLfConvertError, DamlLfConvertResult};
use crate::lf_protobuf::com::daml::daml_lf_1::expr::{
    enum_con, rec_proj, rec_upd, struct_proj, struct_upd, variant_con,
};
use crate::lf_protobuf::com::daml::daml_lf_1::{
    case_alt, def_data_type, def_template, def_type_syn, field_with_expr, field_with_type, module, module_ref, r#type,
    template_choice, type_con_name, type_syn_name, type_var_with_kind, update, var_with_type,
};
use crate::{LanguageFeatureVersion, LanguageVersion};
use std::borrow::Cow;
use std::fmt;
use std::fmt::{Error, Formatter};

/// Resolve Daml package interned dotted names and strings.
pub trait PackageInternedResolver {
    fn package_id(&self) -> &str;
    fn language_version(&self) -> LanguageVersion;
    fn interned_strings(&self) -> &[String];
    fn interned_dotted_names(&self) -> &[&[i32]];
    fn interned_types(&self) -> &[DamlTypePayload<'_>];

    /// Partially resolve an interned dotted name by index to interned string indices.
    fn resolve_dotted_to_indices(&self, index: i32) -> DamlLfConvertResult<&[i32]> {
        Ok(self.interned_dotted_names().get(index as usize).req()?)
    }

    /// Resolve an interned string to a `&str` by index.
    fn resolve_string_raw(&self, index: i32) -> DamlLfConvertResult<&str> {
        Ok(self.interned_strings().get(index as usize).req()?)
    }

    /// Resolve multiple interned strings to a `Vec<&str>` by indices.
    fn resolve_strings_raw(&self, indices: &[i32]) -> DamlLfConvertResult<Vec<&str>> {
        indices.iter().map(|&i| self.resolve_string_raw(i)).collect::<DamlLfConvertResult<_>>()
    }

    /// Fully resolve an interned dotted name to a `Vec<&str>` by index.
    fn resolve_dotted_raw(&self, index: i32) -> DamlLfConvertResult<Vec<&str>> {
        self.resolve_strings_raw(self.resolve_dotted_to_indices(index)?)
    }

    /// Resolve an interned string to a `Cow<str>` by index.
    fn resolve_string(&self, index: i32) -> DamlLfConvertResult<Cow<'_, str>> {
        Ok(Cow::from(self.resolve_string_raw(index)?))
    }

    /// Resolve multiple interned strings to a `Vec<Cow<str>>` by indices.
    fn resolve_strings(&self, indices: &[i32]) -> DamlLfConvertResult<Vec<Cow<'_, str>>> {
        indices.iter().map(|&i| self.resolve_string_raw(i).map(Cow::from)).collect::<DamlLfConvertResult<_>>()
    }

    /// Fully resolve an interned dotted name to a `Vec<Cow<str>>` by index.
    fn resolve_dotted(&self, index: i32) -> DamlLfConvertResult<Vec<Cow<'_, str>>> {
        self.resolve_strings(self.resolve_dotted_to_indices(index)?)
    }

    /// Resolve an interned type to a `DamlTypePayload`
    fn resolve_type(&self, index: i32) -> DamlLfConvertResult<&DamlTypePayload<'_>> {
        self.interned_types().get(index as usize).req()
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
            InternableString::LiteralString(s) => write!(f, "InternableString::Literal({s})"),
            InternableString::InternedString(i) => write!(f, "InternableString::Interned({i})"),
        }
    }
}

impl<'a> InternableString<'a> {
    pub fn resolve(&self, resolver: &'a impl PackageInternedResolver) -> DamlLfConvertResult<Cow<'a, str>> {
        Ok(match *self {
            InternableString::LiteralString(s) => {
                check_does_not_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_STRINGS)?;
                Cow::from(s)
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
    /// Create an `InternableDottedName` implied from the supplied values.
    ///
    /// If the `literal` slice is non-empty then a `LiteralDottedName` variant is returned, otherwise
    /// `InternedDottedName` is returned.  The chosen variant will be validated at resolution time by
    /// `InternableDottedName::resolve()` or `InternableDottedName::resolve_last()` which will report an error if the
    /// wrong variant was implied.
    pub fn new_implied(interned: i32, literal: &'a [String]) -> Self {
        if literal.is_empty() {
            InternableDottedName::InternedDottedName(interned)
        } else {
            InternableDottedName::LiteralDottedName(literal)
        }
    }

    /// Resolve all items in an `InternableDottedName` to a `Vec<Cow<str>>`.
    pub fn resolve(&self, resolver: &'a impl PackageInternedResolver) -> DamlLfConvertResult<Vec<Cow<'a, str>>> {
        Ok(match *self {
            InternableDottedName::LiteralDottedName(dn) => {
                check_does_not_support_feature(
                    resolver.language_version(),
                    &LanguageFeatureVersion::INTERNED_DOTTED_NAMES,
                )?;
                dn.iter().map(Cow::from).collect()
            },
            InternableDottedName::InternedDottedName(i) => {
                check_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_DOTTED_NAMES)?;
                resolver.resolve_dotted(i)?
            },
        })
    }

    /// Resolve the last items in an `InternableDottedName` to a `Cow<'a, str>`.
    pub fn resolve_last(&self, resolver: &'a impl PackageInternedResolver) -> DamlLfConvertResult<Cow<'a, str>> {
        Ok(match *self {
            InternableDottedName::LiteralDottedName(dn) => {
                check_does_not_support_feature(
                    resolver.language_version(),
                    &LanguageFeatureVersion::INTERNED_DOTTED_NAMES,
                )?;
                dn.last().map(Cow::from).req()?
            },
            InternableDottedName::InternedDottedName(i) => {
                check_support_feature(resolver.language_version(), &LanguageFeatureVersion::INTERNED_DOTTED_NAMES)?;
                resolver.resolve_dotted_to_indices(i)?.last().req().and_then(|&s| resolver.resolve_string(s))?
            },
        })
    }

    /// Resolve all items in an `InternableDottedName` to a `Vec<&str>`.
    pub fn resolve_raw(&self, resolver: &'a impl PackageInternedResolver) -> DamlLfConvertResult<Vec<&'a str>> {
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
                resolver.resolve_dotted_raw(i)?
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
            InternableDottedName::InternedDottedName(i) => write!(f, "InternableDottedName::Interned({i})"),
        }
    }
}

/// Make an `<'a> From<thing> for InternableString<'a>` conversion impl.
macro_rules! make_from_internable {
    ($intern:ident $(:: $intern_path:ident)*, $lit_id:ident, $int_id:ident) => {
        impl<'a> From<&'a $intern $(:: $intern_path)*> for InternableString<'a> {
            fn from(field: &'a $intern $(:: $intern_path)*) -> Self {
                match field {
                    $intern $(:: $intern_path)* :: $lit_id(s) => InternableString::LiteralString(s.as_str()),
                    &$intern $(:: $intern_path)* :: $int_id(i) => InternableString::InternedString(i),
                }
            }
        }
    };
}

/// Make an `<'a> From<thing> for InternableDottedName<'a>` conversion impl.
macro_rules! make_from_internable_dotted {
    ($intern:ident $(:: $intern_path:ident)*, $lit_id:ident, $int_id:ident) => {
        impl<'a> From<&'a $intern $(:: $intern_path)*> for InternableDottedName<'a> {
            fn from(field: &'a $intern $(:: $intern_path)*) -> Self {
                match field {
                    $intern $(:: $intern_path)* :: $lit_id(dn) => InternableDottedName::LiteralDottedName(dn.segments.as_slice()),
                    &$intern $(:: $intern_path)* :: $int_id(i) => InternableDottedName::InternedDottedName(i),
                }
            }
        }
    };
}

make_from_internable!(field_with_type::Field, FieldStr, FieldInternedStr);
make_from_internable!(template_choice::Name, NameStr, NameInternedStr);
make_from_internable!(template_choice::SelfBinder, SelfBinderStr, SelfBinderInternedStr);
make_from_internable!(type_var_with_kind::Var, VarStr, VarInternedStr);
make_from_internable!(r#type::var::Var, VarStr, VarInternedStr);
make_from_internable!(field_with_expr::Field, FieldStr, FieldInternedStr);
make_from_internable!(variant_con::VariantCon, VariantConStr, VariantConInternedStr);
make_from_internable!(enum_con::EnumCon, EnumConStr, EnumConInternedStr);
make_from_internable!(rec_proj::Field, FieldStr, FieldInternedStr);
make_from_internable!(struct_proj::Field, FieldStr, FieldInternedStr);
make_from_internable!(rec_upd::Field, FieldStr, FieldInternedStr);
make_from_internable!(update::exercise::Choice, ChoiceStr, ChoiceInternedStr);
make_from_internable!(struct_upd::Field, FieldStr, FieldInternedStr);
make_from_internable!(var_with_type::Var, VarStr, VarInternedStr);
make_from_internable!(case_alt::variant::Variant, VariantStr, VariantInternedStr);
make_from_internable!(case_alt::variant::Binder, BinderStr, BinderInternedStr);
make_from_internable!(case_alt::r#enum::Constructor, ConstructorStr, ConstructorInternedStr);
make_from_internable!(case_alt::cons::VarHead, VarHeadStr, VarHeadInternedStr);
make_from_internable!(case_alt::cons::VarTail, VarTailStr, VarTailInternedStr);
make_from_internable!(case_alt::optional_some::VarBody, VarBodyStr, VarBodyInternedStr);
make_from_internable!(def_template::Param, ParamStr, ParamInternedStr);
make_from_internable_dotted!(def_data_type::Name, NameDname, NameInternedDname);
make_from_internable_dotted!(def_template::Tycon, TyconDname, TyconInternedDname);
make_from_internable_dotted!(module::Name, NameDname, NameInternedDname);
make_from_internable_dotted!(module_ref::ModuleName, ModuleNameDname, ModuleNameInternedDname);
make_from_internable_dotted!(type_con_name::Name, NameDname, NameInternedDname);
make_from_internable_dotted!(type_syn_name::Name, NameDname, NameInternedDname);
make_from_internable_dotted!(def_type_syn::Name, NameDname, NameInternedDname);
