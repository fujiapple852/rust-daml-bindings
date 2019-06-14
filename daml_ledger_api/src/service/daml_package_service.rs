use futures::Future;
use grpcio::Channel;
use grpcio::ClientUnaryReceiver;

use crate::data::package::DamlPackage;
use crate::data::package::DamlPackageStatus;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf_autogen::package_service::GetPackageRequest;
use crate::grpc_protobuf_autogen::package_service::GetPackageResponse;
use crate::grpc_protobuf_autogen::package_service::GetPackageStatusRequest;
use crate::grpc_protobuf_autogen::package_service::GetPackageStatusResponse;
use crate::grpc_protobuf_autogen::package_service::ListPackagesRequest;
use crate::grpc_protobuf_autogen::package_service::ListPackagesResponse;
use crate::grpc_protobuf_autogen::package_service_grpc::PackageServiceClient;

/// Query and extract the DAML LF packages that are supported by the DAML ledger.
pub struct DamlPackageService {
    grpc_client: PackageServiceClient,
    ledger_id: String,
}

impl DamlPackageService {
    pub fn new(channel: Channel, ledger_id: String) -> Self {
        Self {
            grpc_client: PackageServiceClient::new(channel),
            ledger_id,
        }
    }

    pub fn list_packages(&self) -> DamlResult<impl Future<Item = Vec<String>, Error = DamlError>> {
        let mut request = ListPackagesRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        let async_response: ClientUnaryReceiver<ListPackagesResponse> =
            self.grpc_client.list_packages_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|r| r.get_package_ids().iter().map(String::to_string).collect()))
    }

    pub fn list_packages_sync(&self) -> DamlResult<Vec<String>> {
        self.list_packages()?.wait()
    }

    /// TODO fully document this
    pub fn get_package(
        &self,
        package_id: impl Into<String>,
    ) -> DamlResult<impl Future<Item = DamlPackage, Error = DamlError>> {
        self.get_package_with_trace(package_id, None)
    }

    pub fn get_package_sync(&self, package_id: impl Into<String>) -> DamlResult<DamlPackage> {
        self.get_package_with_trace(package_id, None)?.wait()
    }

    pub fn get_package_with_trace_sync(
        &self,
        package_id: impl Into<String>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<DamlPackage> {
        self.get_package_with_trace(package_id, trace_context)?.wait()
    }

    pub fn get_package_with_trace(
        &self,
        package_id: impl Into<String>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Future<Item = DamlPackage, Error = DamlError>> {
        let mut request = GetPackageRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        request.set_package_id(package_id.into());
        if let Some(tc) = trace_context.into() {
            request.set_trace_context(tc.into());
        }
        let async_response: ClientUnaryReceiver<GetPackageResponse> = self.grpc_client.get_package_async(&request)?;
        Ok(async_response.map_err(Into::into).map(Into::into))
    }

    /// TODO fully document this
    pub fn get_package_status(&self, package_id: impl Into<String>) -> DamlResult<DamlPackageStatus> {
        self.get_package_status_with_trace(package_id, None)?.wait()
    }

    pub fn get_package_status_sync(&self, package_id: impl Into<String>) -> DamlResult<DamlPackageStatus> {
        self.get_package_status_with_trace(package_id, None)?.wait()
    }

    pub fn get_package_status_with_trace_sync(
        &self,
        package_id: impl Into<String>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<DamlPackageStatus> {
        self.get_package_status_with_trace(package_id, trace_context)?.wait()
    }

    pub fn get_package_status_with_trace(
        &self,
        package_id: impl Into<String>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Future<Item = DamlPackageStatus, Error = DamlError>> {
        let mut request = GetPackageStatusRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        request.set_package_id(package_id.into());
        if let Some(tc) = trace_context.into() {
            request.set_trace_context(tc.into());
        }
        let async_response: ClientUnaryReceiver<GetPackageStatusResponse> =
            self.grpc_client.get_package_status_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|r| r.get_package_status().into()))
    }
}
