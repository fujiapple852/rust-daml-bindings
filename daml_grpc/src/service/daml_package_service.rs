use crate::data::package::DamlPackage;
use crate::data::package::DamlPackageStatus;

use crate::data::DamlResult;
use crate::data::DamlTraceContext;

use crate::grpc_protobuf::com::daml::ledger::api::v1::package_service_client::PackageServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{
    GetPackageRequest, GetPackageStatusRequest, ListPackagesRequest, PackageStatus, TraceContext,
};
use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use crate::util::Required;
use log::{debug, trace};
use std::convert::TryFrom;
use tonic::transport::Channel;

/// Query and extract the DAML LF packages that are supported by the DAML ledger.
pub struct DamlPackageService {
    channel: Channel,
    ledger_id: String,
    auth_token: Option<String>,
}

impl DamlPackageService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
            auth_token,
        }
    }

    /// DOCME fully document this
    pub async fn list_packages(&self) -> DamlResult<Vec<String>> {
        self.list_packages_with_trace(None).await
    }

    pub async fn list_packages_with_trace(
        &self,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<Vec<String>> {
        debug!("list_packages");
        let payload = ListPackagesRequest {
            ledger_id: self.ledger_id.clone(),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("list_packages payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let all_packages = self.client().list_packages(make_request(payload, &self.auth_token)?).await?;
        Ok(all_packages.into_inner().package_ids)
    }

    /// DOCME fully document this
    pub async fn get_package(&self, package_id: impl Into<String>) -> DamlResult<DamlPackage> {
        self.get_package_with_trace(package_id, None).await
    }

    pub async fn get_package_with_trace(
        &self,
        package_id: impl Into<String>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<DamlPackage> {
        debug!("get_package");
        let payload = GetPackageRequest {
            ledger_id: self.ledger_id.clone(),
            package_id: package_id.into(),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("get_package payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let package = self.client().get_package(make_request(payload, &self.auth_token)?).await?;
        DamlPackage::try_from(package.into_inner())
    }

    /// DOCME fully document this
    pub async fn get_package_status(&self, package_id: impl Into<String>) -> DamlResult<DamlPackageStatus> {
        self.get_package_status_with_trace(package_id, None).await
    }

    pub async fn get_package_status_with_trace(
        &self,
        package_id: impl Into<String>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<DamlPackageStatus> {
        debug!("get_package_status");
        let payload = GetPackageStatusRequest {
            ledger_id: self.ledger_id.clone(),
            package_id: package_id.into(),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("get_package_status payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let package_status = self.client().get_package_status(make_request(payload, &self.auth_token)?).await?;
        Ok(DamlPackageStatus::from(PackageStatus::from_i32(package_status.into_inner().package_status).req()?))
    }

    fn client(&self) -> PackageServiceClient<Channel> {
        PackageServiceClient::new(self.channel.clone())
    }
}

impl DamlTokenRefresh for DamlPackageService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
