#![allow(unused_imports)]
#![allow(unused)]
use anyhow::Result;
use daml::api::{DamlLedgerClientBuilder, DamlSandboxTokenBuilder};
use daml::lf::element::{
    DamlAbsoluteDataRef, DamlArchive, DamlChoice, DamlDataRef, DamlElementVisitor, DamlLocalDataRef,
    DamlNonLocalDataRef, DamlVisitableElement,
};
use daml::lf::{DamlLfArchive, DamlLfArchivePayload, DamlLfHashFunction, DarFile, DarManifest};
use daml::prelude::{DamlError, DamlResult};
use futures::stream::FuturesUnordered;
use futures::TryStreamExt;
use itertools::all;
use prettytable::format;
use prettytable::{color, Attr, Cell, Row, Table};
use std::collections::{HashMap, HashSet};
use std::iter::{once, FromIterator};
use std::sync::Arc;
use tokio::task::JoinHandle;
use uuid::Uuid;

const UNKNOWN_LF_ARCHIVE_PREFIX: &str = "__LF_ARCHIVE_NAME";

fn make_ec256_token(token_key_path: &str) -> Result<String> {
    Ok(DamlSandboxTokenBuilder::new_with_duration_secs(30)
        .admin(true)
        .new_ec256_token(std::fs::read_to_string(token_key_path)?)?)
}

pub(crate) async fn list(uri: &str, token_key_path: Option<&str>) -> Result<()> {
    let ledger_client = Arc::new(match token_key_path {
        Some(key) => DamlLedgerClientBuilder::uri(uri).with_auth(make_ec256_token(key)?).connect().await?,
        None => DamlLedgerClientBuilder::uri(uri).connect().await?,
    });
    let packages = ledger_client.package_management_service().list_known_packages().await?;
    let mut table = Table::new();
    table.set_titles(Row::new(
        vec!["name", "version", "package_id", "lf", "description", "size", "since"]
            .into_iter()
            .map(Cell::new)
            .collect(),
    ));
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    // download all known packages
    let handles: FuturesUnordered<JoinHandle<Result<DamlLfArchive>>> = packages
        .iter()
        .map(|pd| {
            let ledger_client = ledger_client.clone();
            let pid = pd.package_id.clone();
            tokio::spawn(async move {
                let package = ledger_client.package_service().get_package(pid).await?;
                let archive = DamlLfArchivePayload::from_bytes(package.payload)?;
                let main = DamlLfArchive::new(
                    format!("{}-{}", UNKNOWN_LF_ARCHIVE_PREFIX, Uuid::new_v4()),
                    archive,
                    DamlLfHashFunction::SHA256,
                    package.hash,
                );
                Ok(main)
            })
        })
        .collect();

    let mut all_archives: Vec<DamlLfArchive> = handles
        .try_collect::<Vec<Result<DamlLfArchive>>>()
        .await?
        .into_iter()
        .collect::<Result<Vec<DamlLfArchive>>>()?;

    build_dar(all_archives)?;

    // // build a pseudo DarFile with all packages (pick one as main)
    // // TODO some library function for this
    // let manifest = DarManifest::new_implied("", vec!["".to_owned()]);
    // let (first, rest) = all_archives.try_swap_remove(0).map(|i| (i, all_archives)).unwrap();
    // let archive = DarFile::new(manifest, first, rest);
    //
    //
    // let extracted_package_details = archive.apply(|arc| {
    //     arc.packages
    //         .iter()
    //         .map(|(_, p)| {
    //             PackageDisplayInfo::new(
    //                 p.name.to_owned(),
    //                 p.package_id.to_owned(),
    //                 p.version.map(String::from),
    //                 p.language_version.to_string(),
    //             )
    //         })
    //         .collect::<Vec<_>>()
    // })?;
    //
    //

    // let foo: HashMap<String, PackageDisplayInfo> =
    //     extracted_package_details.into_iter().map(|disp| (disp.package_id.clone(), disp)).collect();
    //
    // for package_details in &packages {
    //     let package_id = package_details.package_id.as_str();
    //     let desc = package_details.source_description.as_str();
    //     let known_since = package_details.known_since.to_string();
    //     let package_size = package_details.package_size.to_string();
    //     let display_data = &foo[package_id];
    //     let name = if display_data.name.starts_with(UNKNOWN_LF_ARCHIVE_PREFIX) {
    //         "unknown"
    //     } else {
    //         &display_data.name
    //     };
    //     let version = display_data.version.as_ref().map(String::from).unwrap_or_else(|| "n/a".to_owned());
    //     let language_version = &display_data.language_version;
    //     table.add_row(row(&name, &version, package_id, &language_version, desc, &package_size, &known_since));
    // }
    // table.printstd();
    Ok(())
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
struct PackageInfo {
    package_id: String,
    package_name: String,
}

#[derive(Default)]
struct PackageDependencyVisitor {
    refs: HashSet<PackageInfo>,
}

impl DamlElementVisitor for PackageDependencyVisitor {
    fn pre_visit_non_local_data_ref(&mut self, non_local_data_ref: &DamlNonLocalDataRef<'_>) {
        self.refs.insert(PackageInfo {
            package_id: non_local_data_ref.target_package_id.to_owned(),
            package_name: non_local_data_ref.target_package_name.to_owned(),
        });
    }

    fn pre_visit_absolute_data_ref(&mut self, absolute_data_ref: &DamlAbsoluteDataRef<'_>) {
        self.refs.insert(PackageInfo {
            package_id: absolute_data_ref.package_id.to_owned(),
            package_name: absolute_data_ref.package_name.to_owned(),
        });
    }
}

fn extract_package_dependencies(archive: &DamlArchive, package_info: PackageInfo) -> DamlResult<HashSet<PackageInfo>> {
    let package = archive
        .packages
        .values()
        .find(|&p| p.package_id == package_info.package_id)
        .ok_or_else(|| DamlError::Other("TODO".to_owned()))?;
    let mut visitor = PackageDependencyVisitor {
        refs: Default::default(),
    };
    package.accept(&mut visitor);
    visitor.refs.into_iter().fold(Ok(HashSet::from_iter(vec![package_info].into_iter())), |mut all_refs, name| {
        match all_refs {
            Ok(mut r) => {
                r.extend(extract_package_dependencies(archive, name)?);
                Ok(r)
            },
            Err(e) => Err(e),
        }
    })
}

fn assemble_dar(main: DamlLfArchive, dependencies: Vec<DamlLfArchive>) -> DarFile {
    let manifest = DarManifest::new_implied(main.name.clone(), dependencies.iter().map(|n| n.name.clone()).collect());
    DarFile::new(manifest, main, dependencies)
}

fn build_dar(mut all_archives: Vec<DamlLfArchive>) -> Result<()> {
    // 1: extract an arbitrary package and build temp DarFile with all packages
    let (first, rest) = all_archives.try_swap_remove(0).map(|i| (i, all_archives)).unwrap();
    let everything_dar = assemble_dar(first, rest);

    // 2: determine all dependencies of the stdlib package
    // TODO our package does not seem to reference stdlib so we just have to add our real main package to the dependency
    // list, somehow...
    let dependencies: HashSet<PackageInfo> = everything_dar.apply(|arc| {
        let stdlib = arc
            .packages
            .values()
            .find_map(|a| {
                if a.name == "daml-stdlib" {
                    Some(PackageInfo {
                        package_id: a.package_id.to_owned(),
                        package_name: a.name.to_owned(),
                    })
                } else {
                    None
                }
            })
            .unwrap();
        extract_package_dependencies(arc, stdlib)
    })??;

    // 3: we have to filter and rename each DamlLfArchive with the name we have just extracted from the package
    let dep_map: HashMap<String, String> = dependencies.into_iter().map(|d| (d.package_id, d.package_name)).collect();
    let renamed: Vec<DamlLfArchive> = once(everything_dar.main)
        .chain(everything_dar.dependencies)
        .filter_map(|mut a| {
            dep_map.get_key_value(&a.hash).map(|(k, v)| {
                a.name = v.clone();
                a
            })
        })
        .collect();

    // 4: we can extract the "main" package by name and build the final Dar

    // 5: write out the zip file, somehow

    for x in renamed {
        dbg!(x.name, x.hash);
    }

    // dbg!(dep_map);
    Ok(())
}

struct PackageDisplayInfo {
    name: String,
    package_id: String,
    version: Option<String>,
    language_version: String,
}

impl PackageDisplayInfo {
    pub fn new(
        name: impl Into<String>,
        package_id: impl Into<String>,
        version: impl Into<Option<String>>,
        language_version: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            package_id: package_id.into(),
            version: version.into(),
            language_version: language_version.into(),
        }
    }
}

fn row(
    name: &str,
    version: &str,
    package_id: &str,
    language_version: &str,
    description: &str,
    package_size: &str,
    known_since: &str,
) -> Row {
    Row::new(
        vec![name, version, package_id, language_version, description, package_size, known_since]
            .into_iter()
            .map(cell)
            .collect(),
    )
}

fn cell(data: &str) -> Cell {
    Cell::new(data).with_style(Attr::Bold).with_style(Attr::ForegroundColor(color::WHITE))
}

trait TrySwapRemove<T>: Sized {
    fn try_swap_remove(&mut self, index: usize) -> Option<T>;
}

impl<T> TrySwapRemove<T> for Vec<T> {
    fn try_swap_remove(&mut self, index: usize) -> Option<T> {
        if index < self.len() {
            Some(self.swap_remove(index))
        } else {
            None
        }
    }
}
