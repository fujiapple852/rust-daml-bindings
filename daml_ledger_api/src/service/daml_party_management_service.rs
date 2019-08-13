use futures::Future;
use grpcio::Channel;
use grpcio::ClientUnaryReceiver;

use crate::data::DamlError;
use crate::data::DamlResult;
use crate::grpc_protobuf_autogen::party_management_service::{
    AllocatePartyRequest, AllocatePartyResponse, GetParticipantIdRequest, GetParticipantIdResponse,
    ListKnownPartiesRequest, ListKnownPartiesResponse,
};
use crate::grpc_protobuf_autogen::party_management_service_grpc::PartyManagementServiceClient;

use crate::data::party::DamlPartyDetails;

/// Inspect the party management state of a ledger participant and modify the parts that are modifiable.
///
/// We use 'backing participant' to refer to this specific participant in the methods of this API.
///
/// # Errors
///
/// When the participant is run in mode requiring authentication, all the calls in this interface will respond with
/// UNAUTHENTICATED, if the caller fails to provide a valid access token, and will respond with `PERMISSION_DENIED`, if
/// the claims in the token are insufficient to perform a given operation. Subsequently, only specific errors of
/// individual calls not related to authorization will be described.
pub struct DamlPartyManagementService {
    grpc_client: PartyManagementServiceClient,
}

impl DamlPartyManagementService {
    pub fn new(channel: Channel) -> Self {
        Self {
            grpc_client: PartyManagementServiceClient::new(channel),
        }
    }

    /// Return the identifier of the backing participant.
    ///
    /// All horizontally scaled replicas should return the same id.
    ///
    /// # Errors
    ///
    /// This method is expected to succeed provided the backing participant is healthy, otherwise it responds with
    /// `INTERNAL` grpc error.
    pub fn get_participant_id(&self) -> DamlResult<impl Future<Item = String, Error = DamlError>> {
        let request = GetParticipantIdRequest::new();
        let async_response: ClientUnaryReceiver<GetParticipantIdResponse> =
            self.grpc_client.get_participant_id_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|mut r| r.take_participant_id()))
    }

    /// Synchronous version of `get_participant_id` which blocks on the calling thread.
    ///
    /// See [`get_participant_id`] for details of the behaviour and example usage.
    ///
    /// [`get_participant_id`]: DamlPartyManagementService::get_participant_id
    pub fn get_participant_id_sync(&self) -> DamlResult<String> {
        self.get_participant_id()?.wait()
    }

    /// List the parties known by the backing participant.
    ///
    /// The list returned contains parties whose ledger access is facilitated bb backing participant and the ones
    /// maintained elsewhere.
    pub fn list_known_parties(&self) -> DamlResult<impl Future<Item = Vec<DamlPartyDetails>, Error = DamlError>> {
        let request = ListKnownPartiesRequest::new();
        let async_response: ClientUnaryReceiver<ListKnownPartiesResponse> =
            self.grpc_client.list_known_parties_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|mut r| r.take_party_details().into_iter().map(Into::into).collect()))
    }

    /// Synchronous version of `list_known_parties` which blocks on the calling thread.
    ///
    /// See [`list_known_parties`] for details of the behaviour and example usage.
    ///
    /// [`list_known_parties`]: DamlPartyManagementService::list_known_parties
    pub fn list_known_parties_sync(&self) -> DamlResult<Vec<DamlPartyDetails>> {
        self.list_known_parties()?.wait()
    }

    /// Adds a new party to the set managed by the backing participant.
    ///
    /// Caller specifies a party identifier suggestion, the actual identifier allocated might be different and is
    /// implementation specific.
    ///
    /// # Errors
    ///
    /// This call will either succeed or respond with UNIMPLEMENTED if synchronous party allocation is not supported by
    /// the backing participant.
    pub fn allocate_party(
        &self,
        party_id_hint: impl Into<String>,
        display_name: impl Into<String>,
    ) -> DamlResult<impl Future<Item = DamlPartyDetails, Error = DamlError>> {
        let mut request = AllocatePartyRequest::new();
        request.set_party_id_hint(party_id_hint.into());
        request.set_display_name(display_name.into());
        let async_response: ClientUnaryReceiver<AllocatePartyResponse> =
            self.grpc_client.allocate_party_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|mut r| r.take_party_details().into()))
    }

    /// Synchronous version of `allocate_party` which blocks on the calling thread.
    ///
    /// See [`allocate_party`] for details of the behaviour and example usage.
    ///
    /// [`allocate_party`]: DamlPartyManagementService::allocate_party
    pub fn allocate_party_sync(
        &self,
        party_id_hint: impl Into<String>,
        display_name: impl Into<String>,
    ) -> DamlResult<DamlPartyDetails> {
        self.allocate_party(party_id_hint, display_name)?.wait()
    }
}
