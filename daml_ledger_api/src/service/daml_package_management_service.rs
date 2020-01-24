use crate::data::package::DamlPackageDetails;

use crate::data::DamlResult;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::package_management_service_client::PackageManagementServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::{ListKnownPackagesRequest, UploadDarFileRequest};
use bytes::buf::Buf;
use bytes::Bytes;
use std::convert::TryFrom;
use tonic::transport::Channel;
use tonic::Request;

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
}

impl DamlPackageManagementService {
    pub fn new(channel: Channel) -> Self {
        Self {
            channel,
        }
    }

    /// Returns the details of all DAML-LF packages known to the backing participant.
    pub async fn list_known_packages(&self) -> DamlResult<Vec<DamlPackageDetails>> {
        let request = Request::new(ListKnownPackagesRequest {});
        let all_known_packages = self.client().list_known_packages(request).await?;
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
        let request = Request::new(UploadDarFileRequest {
            dar_file: bytes.into().bytes().to_vec(),
            submission_id: submission_id.unwrap_or_default(),
        });
        self.client().upload_dar_file(request).await.map_err(Into::into).map(|_| ())
    }

    fn client(&self) -> PackageManagementServiceClient<Channel> {
        PackageManagementServiceClient::new(self.channel.clone())
    }
}
