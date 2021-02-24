use crate::data::party::DamlPartyDetails;
use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::party_management_service_client::PartyManagementServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::{
    AllocatePartyRequest, GetParticipantIdRequest, GetPartiesRequest, ListKnownPartiesRequest,
};
use crate::service::common::make_request;
use crate::util::Required;
use std::fmt::Debug;
use tonic::transport::Channel;
use tracing::{instrument, trace};

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
#[derive(Debug)]
pub struct DamlPartyManagementService<'a> {
    channel: Channel,
    auth_token: Option<&'a str>,
}

impl<'a> DamlPartyManagementService<'a> {
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

    /// Return the identifier of the backing participant.
    ///
    /// All horizontally scaled replicas should return the same id.
    ///
    /// # Errors
    ///
    /// This method is expected to succeed provided the backing participant is healthy, otherwise it responds with
    /// `INTERNAL` grpc error.
    #[instrument(skip(self))]
    pub async fn get_participant_id(&self) -> DamlResult<String> {
        let payload = GetParticipantIdRequest {};
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().get_participant_id(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        Ok(response.participant_id)
    }

    /// Get the party details of the given parties.
    ///
    /// Only known parties will be returned in the list.
    #[instrument(skip(self))]
    pub async fn get_parties(&self, parties: impl Into<Vec<String>> + Debug) -> DamlResult<Vec<DamlPartyDetails>> {
        let payload = GetPartiesRequest {
            parties: parties.into(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().get_parties(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        Ok(response.party_details.into_iter().map(DamlPartyDetails::from).collect())
    }

    /// List the parties known by the backing participant.
    ///
    /// The list returned contains parties whose ledger access is facilitated bb backing participant and the ones
    /// maintained elsewhere.
    #[instrument(skip(self))]
    pub async fn list_known_parties(&self) -> DamlResult<Vec<DamlPartyDetails>> {
        let payload = ListKnownPartiesRequest {};
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().list_known_parties(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        Ok(response.party_details.into_iter().map(DamlPartyDetails::from).collect())
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
    #[instrument(skip(self))]
    pub async fn allocate_party(
        &self,
        party_id_hint: impl Into<String> + Debug,
        display_name: impl Into<String> + Debug,
    ) -> DamlResult<DamlPartyDetails> {
        let payload = AllocatePartyRequest {
            party_id_hint: party_id_hint.into(),
            display_name: display_name.into(),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().allocate_party(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        Ok(DamlPartyDetails::from(response.party_details.req()?))
    }

    fn client(&self) -> PartyManagementServiceClient<Channel> {
        PartyManagementServiceClient::new(self.channel.clone())
    }
}
