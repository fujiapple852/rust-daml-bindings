use daml_grpc::data::package::DamlPackage;
use daml_grpc::data::{DamlError, DamlResult};
use daml_grpc::DamlGrpcClient;
use daml_lf::{DamlLfArchive, DamlLfArchivePayload, DamlLfHashFunction, DarFile, DarManifest};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use uuid::Uuid;

/// Convenience methods for working with a collection of [`DamlPackage`].
///
/// In the following example a [`DamlPackages`] is created from all known [`DamlPackage`] on a Daml ledger and then
/// converted into [`DarFile`] using the [`ArchiveAutoNamingStyle::Uuid`] naming style:
///
/// ```no_run
/// # use daml_lf::DarFile;
/// # use daml_grpc::DamlGrpcClientBuilder;
/// # use std::thread;
/// # use daml_util::package::{DamlPackages, ArchiveAutoNamingStyle};
/// # fn main() {
/// # futures::executor::block_on(async {
/// let ledger_client = DamlGrpcClientBuilder::uri("http://127.0.0.1").connect().await.unwrap();
/// let packages = DamlPackages::from_ledger(&ledger_client).await.unwrap();
/// let dar = packages.into_dar(ArchiveAutoNamingStyle::Uuid).unwrap();
/// # })
/// # }
/// ```
#[derive(Debug)]
pub struct DamlPackages {
    packages: Vec<DamlPackage>,
}

impl DamlPackages {
    pub fn new(packages: Vec<DamlPackage>) -> Self {
        Self {
            packages,
        }
    }

    /// Create a [`DamlPackages`] from all known [`DamlPackage`] on a Daml ledger.
    pub async fn from_ledger(ledger_client: &DamlGrpcClient) -> DamlResult<Self> {
        let packages = ledger_client.package_service().list_packages().await?;
        let handles = packages
            .iter()
            .map(|pd| async move { ledger_client.package_service().get_package(pd).await })
            .collect::<FuturesUnordered<_>>();
        let all_packages =
            handles.collect::<Vec<DamlResult<_>>>().await.into_iter().collect::<DamlResult<Vec<DamlPackage>>>()?;
        Ok(Self::new(all_packages))
    }

    /// Return the hash of the [`DamlPackage`] which contains a given module or en error if no such package exists.
    ///
    /// The supplied `module_name` name is assumed to be in `DottedName` format, i.e. `TopModule.SubModule.Module`.
    pub fn find_module(self, module_name: &str) -> DamlResult<String> {
        self.into_payloads()?
            .iter()
            .find(|(_, payload)| payload.contains_module(module_name))
            .map_or_else(|| Err("package could not be found".into()), |(package_id, _)| Ok((*package_id).to_string()))
    }

    /// Package all contained [`DamlPackage`] into a single [`DarFile`].
    ///
    /// Note that an arbitrary package is selected as the main and the remainder are the dependencies.  No attempt is
    /// made to ensure that the dependencies do in fact depend on the main.
    // TODO allow specifying the package id to use as the main package
    // TODO allow filtering packages such that only actual dependencies of main are included
    pub fn into_dar(self, auto_naming_style: ArchiveAutoNamingStyle) -> DamlResult<DarFile> {
        let all_archives = self.into_archives(auto_naming_style)?;
        Self::archives_to_dar(all_archives)
    }

    /// Convert all contained [`DamlPackage`] into [`DamlLfArchive`].
    ///
    /// Note that the created archive is not named.
    pub fn into_archives(self, auto_naming_style: ArchiveAutoNamingStyle) -> DamlResult<Vec<DamlLfArchive>> {
        self.packages
            .into_iter()
            .map(|p| {
                let hash = p.hash().to_owned();
                let payload = Self::package_into_payload(p)?;
                let name = match auto_naming_style {
                    ArchiveAutoNamingStyle::Empty => String::default(),
                    ArchiveAutoNamingStyle::Hash => hash.clone(),
                    ArchiveAutoNamingStyle::Uuid => Uuid::new_v4().to_string(),
                };
                Ok(DamlLfArchive::new(name, payload, DamlLfHashFunction::Sha256, hash))
            })
            .collect()
    }

    /// Convert all contained [`DamlPackage`] into [`DamlLfArchivePayload`].
    pub fn into_payloads(self) -> DamlResult<Vec<(String, DamlLfArchivePayload)>> {
        self.packages
            .into_iter()
            .map(|p| {
                let hash = p.hash().to_owned();
                Self::package_into_payload(p).map(|pl| (hash, pl))
            })
            .collect::<DamlResult<Vec<_>>>()
    }

    fn package_into_payload(package: DamlPackage) -> DamlResult<DamlLfArchivePayload> {
        DamlLfArchivePayload::from_bytes(package.take_payload()).map_err(|e| DamlError::Other(e.to_string()))
    }

    fn archives_to_dar(mut all_packages: Vec<DamlLfArchive>) -> DamlResult<DarFile> {
        if all_packages.is_empty() {
            Err("expected at least one archive".into())
        } else {
            let (first, rest) = all_packages.try_swap_remove(0).map(|removed| (removed, all_packages)).unwrap();
            let manifest = DarManifest::new_implied(first.name.clone(), rest.iter().map(|n| n.name.clone()).collect());
            Ok(DarFile::new(manifest, first, rest))
        }
    }
}

/// The automatic naming style to use when creating a `DamlLfArchive` from an unnamed `DamlPackage`.
#[derive(Clone, Copy, Debug)]
pub enum ArchiveAutoNamingStyle {
    /// Name the `DamlLfArchive` with an empty String.
    Empty,
    /// Name the `DamlLfArchive` with the archive hash.
    Hash,
    /// Name the `DamlLfArchive` with a `uuid`.
    Uuid,
}

/// Return the id of a package which contains a given module name or en error if no such package exists.
///
/// The supplied `module_name` name is assumed to be in `DottedName` format, i.e. `TopModule.SubModule.Module`.
pub async fn find_module_package_id(ledger_client: &DamlGrpcClient, module_name: &str) -> DamlResult<String> {
    DamlPackages::from_ledger(ledger_client).await?.find_module(module_name)
}

trait TrySwapRemove<T>: Sized {
    fn try_swap_remove(&mut self, index: usize) -> Option<T>;
}

impl<T> TrySwapRemove<T> for Vec<T> {
    fn try_swap_remove(&mut self, index: usize) -> Option<T> {
        (index < self.len()).then(|| self.swap_remove(index))
    }
}
