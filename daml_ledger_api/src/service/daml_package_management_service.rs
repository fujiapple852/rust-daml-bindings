use crate::data::package::DamlPackageDetails;

use crate::data::DamlResult;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::package_management_service_client::PackageManagementServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::{ListKnownPackagesRequest, UploadDarFileRequest};
use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use bytes::buf::Buf;
use bytes::Bytes;
use log::{debug, trace};
use std::convert::TryFrom;
use tonic::transport::Channel;

/// Query the DAML-LF packages supported by the ledger participant and upload DAR files.
///
/// We use 'backing participant' to refer to this specific participant in the methods of this API.
///
/// # Errors
///
/// When the participant is run in mode requiring authentication, all the calls in this interface will respond with
/// `UNAUTHENTICATED`, if the caller fails to provide a valid access token, and will respond with `PERMISSION_DENIED`,
/// if the claims in the token are insufficient to perform a given operation.
pub struct DamlPackageManagementService {
    channel: Channel,
    auth_token: Option<String>,
}

impl DamlPackageManagementService {
    pub fn new(channel: Channel, auth_token: Option<String>) -> Self {
        Self {
            channel,
            auth_token,
        }
    }

    /// Returns the details of all DAML-LF packages known to the backing participant.
    pub async fn list_known_packages(&self) -> DamlResult<Vec<DamlPackageDetails>> {
        debug!("list_known_packages");
        let payload = ListKnownPackagesRequest {};
        trace!("list_known_packages payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let all_known_packages = self.client().list_known_packages(make_request(payload, &self.auth_token)?).await?;
        all_known_packages.into_inner().package_details.into_iter().map(DamlPackageDetails::try_from).collect()
    }

    /// Upload a DAR file to the backing participant.
    ///
    /// Depending on the ledger implementation this might also make the package available on the whole ledger. This call
    /// might not be supported by some ledger implementations.
    ///
    /// # Errors
    ///
    /// This method will return `UNIMPLEMENTED`, if `dar` package uploading is not supported by the backing
    /// participant.
    ///
    /// If DAR file is too big or is malformed, the backing participant will respond with
    /// `INVALID_ARGUMENT`.
    ///
    /// The maximum supported size is implementation specific.  Contains a DAML archive `dar`file, which in turn is a
    /// jar like zipped container for `daml_lf` archives.
    pub async fn upload_dar_file(&self, bytes: impl Into<Bytes>, submission_id: Option<String>) -> DamlResult<()> {
        debug!("upload_dar_file");
        let payload = UploadDarFileRequest {
            dar_file: bytes.into().bytes().to_vec(),
            submission_id: submission_id.unwrap_or_default(),
        };
        trace!("upload_dar_file payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client().upload_dar_file(make_request(payload, &self.auth_token)?).await.map_err(Into::into).map(|_| ())
    }

    fn client(&self) -> PackageManagementServiceClient<Channel> {
        PackageManagementServiceClient::new(self.channel.clone())
    }
}

impl DamlTokenRefresh for DamlPackageManagementService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
