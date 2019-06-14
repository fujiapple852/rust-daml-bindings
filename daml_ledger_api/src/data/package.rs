use crate::grpc_protobuf_autogen::package_service::GetPackageResponse;
use crate::grpc_protobuf_autogen::package_service::HashFunction;
use crate::grpc_protobuf_autogen::package_service::PackageStatus;

#[derive(Debug, Eq, PartialEq)]
pub struct DamlPackage {
    payload: Vec<u8>,
    hash: String,
    hash_function: DamlHashFunction,
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
