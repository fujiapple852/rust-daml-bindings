use crate::aliases::{BridgeConfig, GrpcClient};
use crate::handler::common::{internal_server_error, parse_auth_header, JsonResult};
use bytes::Bytes;
use daml_json::request::{DamlJsonListPackagesResponse, DamlJsonUploadDarResponse};

/// DOCME
pub struct PackagesHandler {
    _config: BridgeConfig,
    client: GrpcClient,
}

impl PackagesHandler {
    pub fn new(_config: BridgeConfig, client: GrpcClient) -> Self {
        Self {
            _config,
            client,
        }
    }

    /// DOCME
    pub async fn list_all_packages(&self, auth_header: Option<&str>) -> JsonResult<DamlJsonListPackagesResponse> {
        let all_package_ids = self.execute_list_all(auth_header).await?;
        Ok(make_list_all_packages_response(all_package_ids))
    }

    /// DOCME
    pub async fn get_package(&self, package_id: &str, auth_header: Option<&str>) -> JsonResult<Vec<u8>> {
        Ok(self.execute_get(package_id, auth_header).await?)
    }

    /// DOCME
    pub async fn upload_dar(&self, payload: Bytes, auth_header: Option<&str>) -> JsonResult<DamlJsonUploadDarResponse> {
        self.execute_upload(payload, auth_header).await?;
        Ok(make_upload_dar_response())
    }

    async fn execute_list_all(&self, auth_header: Option<&str>) -> JsonResult<Vec<String>> {
        let (token, _) = parse_auth_header(auth_header)?;
        self.client.package_service().with_token(token).list_packages().await.map_err(internal_server_error)
    }

    async fn execute_get(&self, package_id: &str, auth_header: Option<&str>) -> JsonResult<Vec<u8>> {
        let (token, _) = parse_auth_header(auth_header)?;
        Ok(self
            .client
            .package_service()
            .with_token(token)
            .get_package(package_id)
            .await
            .map_err(internal_server_error)?
            .take_payload())
    }

    async fn execute_upload(&self, payload: Bytes, auth_header: Option<&str>) -> JsonResult<()> {
        let (token, _) = parse_auth_header(auth_header)?;
        self.client
            .package_management_service()
            .with_token(token)
            .upload_dar_file(payload, None)
            .await
            .map_err(internal_server_error)
    }
}

const fn make_list_all_packages_response(all_package_ids: Vec<String>) -> DamlJsonListPackagesResponse {
    DamlJsonListPackagesResponse {
        status: 200,
        result: all_package_ids,
        warnings: None,
    }
}

const fn make_upload_dar_response() -> DamlJsonUploadDarResponse {
    DamlJsonUploadDarResponse {
        status: 200,
        result: 1,
        warnings: None,
    }
}
