use crate::convert::archive::wrapper::payload::util::Required;
use crate::convert::archive::wrapper::payload::{DamlModulePayload, PackageInternedResolver};
use crate::convert::error::{DamlCodeGenError, DamlCodeGenResult};
use daml_lf::{DamlLfArchive, DamlLfPackage};
use daml_lf::{LanguageFeatureVersion, LanguageVersion};
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct DamlPackagePayload<'a> {
    pub name: String,
    pub version: Option<String>,
    pub language_version: LanguageVersion,
    pub package_id: &'a str,
    pub interned_strings: &'a [String],
    pub interned_dotted_names: Vec<&'a [i32]>,
    pub modules: HashMap<String, DamlModulePayload<'a>>,
}

impl<'a> PackageInternedResolver for DamlPackagePayload<'a> {
    fn package_id(&self) -> &str {
        self.package_id
    }

    fn language_version(&self) -> LanguageVersion {
        self.language_version
    }

    fn interned_strings(&self) -> &[String] {
        self.interned_strings
    }

    fn interned_dotted_names(&self) -> &[&[i32]] {
        &self.interned_dotted_names
    }
}

impl<'a> PartialEq for DamlPackagePayload<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a> DamlPackagePayload<'a> {
    pub fn module_by_name(&self, module_path: &str) -> Option<&DamlModulePayload<'_>> {
        self.modules.get(module_path)
    }
}

impl<'a> TryFrom<&'a DamlLfArchive> for DamlPackagePayload<'a> {
    type Error = DamlCodeGenError;

    fn try_from(daml_lf_archive: &'a DamlLfArchive) -> DamlCodeGenResult<Self> {
        // A temporary resolver for resolving interned strings before the DamlPackagePayload has been constructed.
        struct SelfResolver<'a> {
            pub language_version: LanguageVersion,
            pub package_id: &'a str,
            pub interned_strings: &'a [String],
            pub interned_dotted_names: &'a [&'a [i32]],
        }

        impl<'a> SelfResolver<'a> {
            pub fn new(
                language_version: LanguageVersion,
                package_id: &'a str,
                interned_strings: &'a [String],
                interned_dotted_names: &'a [&'a [i32]],
            ) -> Self {
                Self {
                    language_version,
                    package_id,
                    interned_strings,
                    interned_dotted_names,
                }
            }
        }

        impl<'a> PackageInternedResolver for SelfResolver<'a> {
            fn package_id(&self) -> &str {
                self.package_id
            }

            fn language_version(&self) -> LanguageVersion {
                self.language_version
            }

            fn interned_strings(&self) -> &[String] {
                self.interned_strings
            }

            fn interned_dotted_names(&self) -> &[&[i32]] {
                self.interned_dotted_names
            }
        }

        Ok(match &daml_lf_archive.payload.package {
            DamlLfPackage::V1(package) => {
                let language_version = daml_lf_archive.payload.language_version;
                let package_id = daml_lf_archive.hash.as_str();
                let interned_strings = package.interned_strings.as_slice();
                let interned_dotted_names: Vec<_> =
                    package.interned_dotted_names.iter().map(|dn| dn.segments_interned_str.as_slice()).collect();
                let self_resolver =
                    SelfResolver::new(language_version, package_id, interned_strings, &interned_dotted_names);
                let (name, version) = if language_version.supports_feature(&LanguageFeatureVersion::PACKAGE_METADATA) {
                    let metadata = package.metadata.as_ref().req()?;
                    let meta_name = self_resolver.resolve_string(metadata.name_interned_str)?;
                    let meta_version = self_resolver.resolve_string(metadata.version_interned_str)?;
                    (meta_name.to_owned(), Some(meta_version.to_owned()))
                } else {
                    let name = daml_lf_archive.name.as_str();
                    (name.to_owned(), None)
                };
                let modules = package
                    .modules
                    .iter()
                    .flat_map(DamlModulePayload::try_from)
                    .map(|m| Ok((m.path.resolve(&self_resolver)?.join("."), m)))
                    .collect::<DamlCodeGenResult<_>>()?;
                Self {
                    name,
                    version,
                    language_version,
                    package_id,
                    interned_strings,
                    interned_dotted_names,
                    modules,
                }
            },
        })
    }
}
