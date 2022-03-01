use std::convert::TryFrom;
use std::fmt::Debug;

use tonic::transport::Channel;
use tracing::{instrument, trace};

use crate::data::package::DamlPackage;
use crate::data::package::DamlPackageStatus;
use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::package_service_client::PackageServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{
    GetPackageRequest, GetPackageStatusRequest, ListPackagesRequest, PackageStatus,
};
use crate::service::common::make_request;
use crate::util::Required;

/// Query and extract the Daml LF packages that are supported by the Daml ledger.
#[derive(Debug)]
pub struct DamlPackageService<'a> {
    channel: Channel,
    ledger_id: &'a str,
    auth_token: Option<&'a str>,
}

impl<'a> DamlPackageService<'a> {
    pub fn new(channel: Channel, ledger_id: &'a str, auth_token: Option<&'a str>) -> Self {
        Self {
            channel,
            ledger_id,
            auth_token,
        }
    }

    /// Override the JWT token to use for this service.
    pub fn with_token(self, auth_token: &'a str) -> Self {
        Self {
            auth_token: Some(auth_token),
            ..self
        }
    }

    /// Override the ledger id to use for this service.
    pub fn with_ledger_id(self, ledger_id: &'a str) -> Self {
        Self {
            ledger_id,
            ..self
        }
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn list_packages(&self) -> DamlResult<Vec<String>> {
        let payload = ListPackagesRequest {
            ledger_id: self.ledger_id.to_owned(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self.client().list_packages(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        Ok(response.package_ids)
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_package(&self, package_id: impl Into<String> + Debug) -> DamlResult<DamlPackage> {
        let payload = GetPackageRequest {
            ledger_id: self.ledger_id.to_owned(),
            package_id: package_id.into(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self.client().get_package(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        DamlPackage::try_from(response)
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_package_status(&self, package_id: impl Into<String> + Debug) -> DamlResult<DamlPackageStatus> {
        let payload = GetPackageStatusRequest {
            ledger_id: self.ledger_id.to_owned(),
            package_id: package_id.into(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self.client().get_package_status(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        Ok(DamlPackageStatus::from(PackageStatus::from_i32(response.package_status).req()?))
    }

    fn client(&self) -> PackageServiceClient<Channel> {
        PackageServiceClient::new(self.channel.clone())
    }
}
