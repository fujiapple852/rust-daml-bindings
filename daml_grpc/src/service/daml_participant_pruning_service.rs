use log::{debug, trace};
use tonic::transport::Channel;

use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::participant_pruning_service_client::ParticipantPruningServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::admin::PruneRequest;
use crate::service::common::make_request;

/// Prunes/truncates the "oldest" transactions from the participant (the participant Ledger Api Server plus any
/// other participant-local state) by removing a portion of the ledger in such a way that the set of future,
/// allowed commands are not affected.
pub struct DamlParticipantPruningService<'a> {
    channel: Channel,
    auth_token: Option<&'a str>,
}

impl<'a> DamlParticipantPruningService<'a> {
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

    /// Prune the ledger specifying the offset before and at which ledger transactions should be removed. Only returns
    /// when the potentially long-running prune request ends successfully or fails.
    pub async fn prune(
        &self,
        prune_up_to: impl Into<String>,
        submission_id: impl Into<Option<String>>,
    ) -> DamlResult<()> {
        debug!("prune");
        let payload = PruneRequest {
            prune_up_to: prune_up_to.into(),
            submission_id: submission_id.into().unwrap_or_default(),
        };
        trace!("prune payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client().prune(make_request(payload, self.auth_token.as_deref())?).await?;
        Ok(())
    }

    fn client(&self) -> ParticipantPruningServiceClient<Channel> {
        ParticipantPruningServiceClient::new(self.channel.clone())
    }
}
