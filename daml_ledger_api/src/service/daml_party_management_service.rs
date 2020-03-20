use crate::data::DamlResult;

use crate::data::party::DamlPartyDetails;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::party_management_service_client::PartyManagementServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::admin::{
    AllocatePartyRequest, GetParticipantIdRequest, GetPartiesRequest, ListKnownPartiesRequest,
};
use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use crate::util::Required;
use log::{debug, trace};
use tonic::transport::Channel;

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
    auth_token: Option<String>,
}

impl DamlPartyManagementService {
    pub fn new(channel: Channel, auth_token: Option<String>) -> Self {
        Self {
            channel,
            auth_token,
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
        debug!("get_participant_id");
        let payload = GetParticipantIdRequest {};
        trace!("get_participant_id payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let participant = self.client().get_participant_id(make_request(payload, &self.auth_token)?).await?;
        Ok(participant.into_inner().participant_id)
    }

    /// Get the party details of the given parties.
    ///
    /// Only known parties will be returned in the list.
    pub async fn get_parties(&self, parties: impl Into<Vec<String>>) -> DamlResult<Vec<DamlPartyDetails>> {
        debug!("get_parties");
        let payload = GetPartiesRequest {
            parties: parties.into(),
        };
        trace!("get_parties payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let parties = self.client().get_parties(make_request(payload, &self.auth_token)?).await?;
        Ok(parties.into_inner().party_details.into_iter().map(DamlPartyDetails::from).collect())
    }

    /// List the parties known by the backing participant.
    ///
    /// The list returned contains parties whose ledger access is facilitated bb backing participant and the ones
    /// maintained elsewhere.
    pub async fn list_known_parties(&self) -> DamlResult<Vec<DamlPartyDetails>> {
        debug!("list_known_parties");
        let payload = ListKnownPartiesRequest {};
        trace!("list_known_parties payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let all_known_parties = self.client().list_known_parties(make_request(payload, &self.auth_token)?).await?;
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
        debug!("allocate_party");
        let payload = AllocatePartyRequest {
            party_id_hint: party_id_hint.into(),
            display_name: display_name.into(),
        };
        trace!("allocate_party payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let allocated_parties = self.client().allocate_party(make_request(payload, &self.auth_token)?).await?;
        Ok(DamlPartyDetails::from(allocated_parties.into_inner().party_details.req()?))
    }

    fn client(&self) -> PartyManagementServiceClient<Channel> {
        PartyManagementServiceClient::new(self.channel.clone())
    }
}

impl DamlTokenRefresh for DamlPartyManagementService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
