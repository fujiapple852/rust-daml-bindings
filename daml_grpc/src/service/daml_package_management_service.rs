use std::convert::TryFrom;
use std::fmt::Debug;

use bytes::Bytes;
use tonic::transport::Channel;
use tracing::{instrument, trace};

use crate::data::package::DamlPackageDetails;
use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::package_management_service_client::PackageManagementServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::{ListKnownPackagesRequest, UploadDarFileRequest};
use crate::service::common::make_request;

/// Query the DAML-LF packages supported by the ledger participant and upload DAR files.
///
/// We use 'backing participant' to refer to this specific participant in the methods of this API.
///
/// # Errors
///
/// When the participant is run in mode requiring authentication, all the calls in this interface will respond with
/// `UNAUTHENTICATED`, if the caller fails to provide a valid access token, and will respond with `PERMISSION_DENIED`,
/// if the claims in the token are insufficient to perform a given operation.
#[derive(Debug)]
pub struct DamlPackageManagementService<'a> {
    channel: Channel,
    auth_token: Option<&'a str>,
}

impl<'a> DamlPackageManagementService<'a> {
    pub fn new(channel: Channel, auth_token: Option<&'a str>) -> Self {
        Self {
            channel,
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

    /// Returns the details of all DAML-LF packages known to the backing participant.
    #[instrument(skip(self))]
    pub async fn list_known_packages(&self) -> DamlResult<Vec<DamlPackageDetails>> {
        let payload = ListKnownPackagesRequest {};
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self.client().list_known_packages(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        response.package_details.into_iter().map(DamlPackageDetails::try_from).collect()
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
    #[instrument(skip(self))]
    pub async fn upload_dar_file(
        &self,
        bytes: impl Into<Bytes> + Debug,
        submission_id: Option<String>,
    ) -> DamlResult<()> {
        let payload = UploadDarFileRequest {
            dar_file: bytes.into().to_vec(),
            submission_id: submission_id.unwrap_or_default(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        self.client().upload_dar_file(make_request(payload, self.auth_token)?).await.map_err(Into::into).map(|_| ())
    }

    fn client(&self) -> PackageManagementServiceClient<Channel> {
        PackageManagementServiceClient::new(self.channel.clone())
    }
}
