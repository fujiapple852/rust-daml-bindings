use crate::data::package::DamlPackage;
use crate::data::package::DamlPackageStatus;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf::com::daml::ledger::api::v1::package_service_client::PackageServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{
    GetPackageRequest, GetPackageStatusRequest, ListPackagesRequest, PackageStatus, TraceContext,
};
use crate::service::common::make_request;
use crate::util::Required;
use log::{debug, trace};
use std::convert::TryFrom;
use tonic::transport::Channel;

/// Query and extract the DAML LF packages that are supported by the DAML ledger.
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
    pub async fn list_packages(&self) -> DamlResult<Vec<String>> {
        self.list_packages_with_trace(None).await
    }

    pub async fn list_packages_with_trace(
        &self,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<Vec<String>> {
        debug!("list_packages");
        let payload = ListPackagesRequest {
            ledger_id: self.ledger_id.to_owned(),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("list_packages payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let all_packages = self.client().list_packages(make_request(payload, self.auth_token.as_deref())?).await?;
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
            ledger_id: self.ledger_id.to_owned(),
            package_id: package_id.into(),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("get_package payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let package = self.client().get_package(make_request(payload, self.auth_token.as_deref())?).await?;
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
            ledger_id: self.ledger_id.to_owned(),
            package_id: package_id.into(),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("get_package_status payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let package_status =
            self.client().get_package_status(make_request(payload, self.auth_token.as_deref())?).await?;
        Ok(DamlPackageStatus::from(PackageStatus::from_i32(package_status.into_inner().package_status).req()?))
    }

    fn client(&self) -> PackageServiceClient<Channel> {
        PackageServiceClient::new(self.channel.clone())
    }
}
