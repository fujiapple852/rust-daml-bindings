use crate::data::package::DamlPackageDetails;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::grpc_protobuf_autogen::package_management_service::{
    ListKnownPackagesRequest, ListKnownPackagesResponse, UploadDarFileRequest, UploadDarFileResponse,
};
use crate::grpc_protobuf_autogen::package_management_service_grpc::PackageManagementServiceClient;
use bytes::buf::Buf;
use bytes::IntoBuf;
use futures::Future;
use grpcio::Channel;
use grpcio::ClientUnaryReceiver;

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
    grpc_client: PackageManagementServiceClient,
}

impl DamlPackageManagementService {
    pub fn new(channel: Channel) -> Self {
        Self {
            grpc_client: PackageManagementServiceClient::new(channel),
        }
    }

    /// Returns the details of all DAML-LF packages known to the backing participant.
    pub fn list_known_packages(&self) -> DamlResult<impl Future<Item = Vec<DamlPackageDetails>, Error = DamlError>> {
        let request = ListKnownPackagesRequest::new();
        let async_response: ClientUnaryReceiver<ListKnownPackagesResponse> =
            self.grpc_client.list_known_packages_async(&request)?;
        Ok(async_response
            .map_err(Into::into)
            .map(|mut r| r.take_package_details().into_iter().map(Into::into).collect()))
    }

    /// Synchronous version of `list_known_packages` which blocks on the calling thread.
    ///
    /// See [`list_known_packages`] for details of the behaviour and example usage.
    ///
    /// [`list_known_packages`]: DamlPackageManagementService::list_known_packages
    pub fn list_known_packages_sync(&self) -> DamlResult<Vec<DamlPackageDetails>> {
        self.list_known_packages()?.wait()
    }

    /// Upload a DAR file to the backing participant.
    ///
    /// Depending on the ledger implementation this might also make the package available on the whole ledger. This call
    /// might not be supported by some ledger implementations.
    ///
    /// # Errors
    ///
    /// This method will return `UNIMPLEMENTED`, if `dar` package uploading is not supported by the backing
    /// participant. If DAR file is too big or is malformed, the backing participant will respond with
    /// `INVALID_ARGUMENT`.  The maximum supported size is implementation specific.  Contains a DAML archive
    /// `dar`file, which in turn is a jar like zipped container for `daml_lf` archives.
    pub fn upload_dar_file(&self, bytes: impl IntoBuf) -> DamlResult<impl Future<Item = (), Error = DamlError>> {
        let mut request = UploadDarFileRequest::new();
        request.set_dar_file(bytes.into_buf().bytes().to_vec());
        let async_response: ClientUnaryReceiver<UploadDarFileResponse> =
            self.grpc_client.upload_dar_file_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|_| ()))
    }

    /// Synchronous version of `upload_dar_file` which blocks on the calling thread.
    ///
    /// See [`upload_dar_file`] for details of the behaviour and example usage.
    ///
    /// [`upload_dar_file`]: DamlPackageManagementService::upload_dar_file
    pub fn upload_dar_file_sync(&self, bytes: impl IntoBuf) -> DamlResult<()> {
        self.upload_dar_file(bytes)?.wait()
    }
}
