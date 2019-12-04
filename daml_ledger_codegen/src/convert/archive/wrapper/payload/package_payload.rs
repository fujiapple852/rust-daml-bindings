use crate::convert::archive::wrapper::payload::*;
use daml_lf::LanguageVersion;
use daml_lf::{DamlLfArchive, DamlLfPackage};
use std::collections::HashMap;

#[derive(Debug)]
pub struct DamlPackagePayload<'a> {
    pub name: &'a str,
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
    pub fn module_by_name(&self, module_path: &str) -> Option<&DamlModulePayload> {
        self.modules.get(module_path)
    }
}

impl<'a> From<&'a DamlLfArchive> for DamlPackagePayload<'a> {
    fn from(daml_lf_archive: &'a DamlLfArchive) -> Self {
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
                &self.interned_dotted_names
            }
        }

        match &daml_lf_archive.payload.package {
            DamlLfPackage::V1(package) => {
                let name = daml_lf_archive.name.as_str();
                let language_version = daml_lf_archive.payload.language_version;
                let package_id = daml_lf_archive.hash.as_str();
                let interned_strings = package.interned_strings.as_slice();
                let interned_dotted_names: Vec<_> =
                    package.interned_dotted_names.iter().map(|dn| dn.segments_interned_str.as_slice()).collect();
                let self_resolver =
                    SelfResolver::new(language_version, package_id, interned_strings, &interned_dotted_names);
                let modules = package
                    .modules
                    .iter()
                    .map(DamlModulePayload::from)
                    .map(|m| (m.path.resolve(&self_resolver).join("."), m))
                    .collect();
                Self {
                    name,
                    language_version,
                    package_id,
                    interned_strings,
                    interned_dotted_names,
                    modules,
                }
            },
        }
    }
}
