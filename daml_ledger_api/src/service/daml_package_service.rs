use crate::data::package::DamlPackage;
use crate::data::package::DamlPackageStatus;

use crate::data::DamlResult;
use crate::data::DamlTraceContext;

use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::package_service_client::PackageServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{
    GetPackageRequest, GetPackageStatusRequest, ListPackagesRequest, PackageStatus, TraceContext,
};
use crate::util::Required;
use std::convert::TryFrom;
use tonic::transport::Channel;
use tonic::Request;

/// Query and extract the DAML LF packages that are supported by the DAML ledger.
pub struct DamlPackageService {
    channel: Channel,
    ledger_id: String,
}

impl DamlPackageService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
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
        let request = Request::new(ListPackagesRequest {
            ledger_id: self.ledger_id.clone(),
            trace_context: trace_context.into().map(TraceContext::from),
        });
        let all_packages = self.client().list_packages(request).await?;
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
        let request = Request::new(GetPackageRequest {
            ledger_id: self.ledger_id.clone(),
            package_id: package_id.into(),
            trace_context: trace_context.into().map(TraceContext::from),
        });
        let package = self.client().get_package(request).await?;
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
        let request = Request::new(GetPackageStatusRequest {
            ledger_id: self.ledger_id.clone(),
            package_id: package_id.into(),
            trace_context: trace_context.into().map(TraceContext::from),
        });
        let package_status = self.client().get_package_status(request).await?;
        Ok(DamlPackageStatus::from(PackageStatus::from_i32(package_status.into_inner().package_status).req()?))
    }

    fn client(&self) -> PackageServiceClient<Channel> {
        PackageServiceClient::new(self.channel.clone())
    }
}
