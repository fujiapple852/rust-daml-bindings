use crate::grpc_protobuf_autogen::package_management_service::PackageDetails;
use crate::grpc_protobuf_autogen::package_service::GetPackageResponse;
use crate::grpc_protobuf_autogen::package_service::HashFunction;
use crate::grpc_protobuf_autogen::package_service::PackageStatus;
use crate::util;
use chrono::{DateTime, Utc};

#[derive(Debug, Eq, PartialEq)]
pub struct DamlPackage {
    pub payload: Vec<u8>,
    pub hash: String,
    pub hash_function: DamlHashFunction,
}

impl DamlPackage {
    pub fn new(
        payload: impl Into<Vec<u8>>,
        hash: impl Into<String>,
        hash_function: impl Into<DamlHashFunction>,
    ) -> Self {
        Self {
            payload: payload.into(),
            hash: hash.into(),
            hash_function: hash_function.into(),
        }
    }

    pub fn payload(&self) -> &Vec<u8> {
        &self.payload
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn hash_function(&self) -> &DamlHashFunction {
        &self.hash_function
    }
}

impl From<GetPackageResponse> for DamlPackage {
    fn from(mut response: GetPackageResponse) -> Self {
        Self::new(response.take_archive_payload(), response.take_hash(), response.get_hash_function())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum DamlPackageStatus {
    Unknown,
    Registered,
}

impl From<PackageStatus> for DamlPackageStatus {
    fn from(status: PackageStatus) -> Self {
        match status {
            PackageStatus::UNKNOWN => DamlPackageStatus::Unknown,
            PackageStatus::REGISTERED => DamlPackageStatus::Registered,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum DamlHashFunction {
    SHA256,
}

impl From<HashFunction> for DamlHashFunction {
    fn from(hash_function: HashFunction) -> Self {
        match hash_function {
            HashFunction::SHA256 => DamlHashFunction::SHA256,
        }
    }
}

/// Detailed information about a DAML `dar` package.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlPackageDetails {
    pub package_id: String,
    pub package_size: u64,
    pub known_since: DateTime<Utc>,
    pub source_description: String,
}

impl DamlPackageDetails {
    pub fn new(
        package_id: impl Into<String>,
        package_size: impl Into<u64>,
        known_since: impl Into<DateTime<Utc>>,
        source_description: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            package_size: package_size.into(),
            known_since: known_since.into(),
            source_description: source_description.into(),
        }
    }

    /// The identity of the DAML-LF package.
    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    /// Size of the package in bytes.
    pub fn package_size(&self) -> u64 {
        self.package_size
    }

    /// Indicates since when the package is known to the backing participant.
    pub fn known_since(&self) -> &DateTime<Utc> {
        &self.known_since
    }

    /// Description provided by the backing participant describing where it got the package from.
    pub fn source_description(&self) -> &str {
        &self.source_description
    }
}

impl From<PackageDetails> for DamlPackageDetails {
    fn from(mut details: PackageDetails) -> Self {
        Self::new(
            details.take_package_id(),
            details.get_package_size(),
            util::make_datetime(&details.take_known_since()),
            details.take_source_description(),
        )
    }
}
