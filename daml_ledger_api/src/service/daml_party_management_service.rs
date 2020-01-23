use crate::data::DamlResult;

use crate::data::party::DamlPartyDetails;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::party_management_service_client::PartyManagementServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::{
    AllocatePartyRequest, GetParticipantIdRequest, ListKnownPartiesRequest,
};
use crate::util::Required;
use tonic::transport::Channel;
use tonic::Request;

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
    channel: Channel,
}

impl DamlPartyManagementService {
    pub fn new(channel: Channel) -> Self {
        Self {
            channel,
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
    pub async fn get_participant_id(&self) -> DamlResult<String> {
        let request = Request::new(GetParticipantIdRequest {});
        let participant = self.client().get_participant_id(request).await?;
        Ok(participant.into_inner().participant_id)
    }

    /// List the parties known by the backing participant.
    ///
    /// The list returned contains parties whose ledger access is facilitated bb backing participant and the ones
    /// maintained elsewhere.
    pub async fn list_known_parties(&self) -> DamlResult<Vec<DamlPartyDetails>> {
        let request = Request::new(ListKnownPartiesRequest {});
        let all_known_parties = self.client().list_known_parties(request).await?;
        Ok(all_known_parties.into_inner().party_details.into_iter().map(DamlPartyDetails::from).collect())
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
    pub async fn allocate_party(
        &self,
        party_id_hint: impl Into<String>,
        display_name: impl Into<String>,
    ) -> DamlResult<DamlPartyDetails> {
        let request = Request::new(AllocatePartyRequest {
            party_id_hint: party_id_hint.into(),
            display_name: display_name.into(),
        });
        let allocated_parties = self.client().allocate_party(request).await?;
        Ok(DamlPartyDetails::from(allocated_parties.into_inner().party_details.req()?))
    }

    fn client(&self) -> PartyManagementServiceClient<Channel> {
        PartyManagementServiceClient::new(self.channel.clone())
    }
}
