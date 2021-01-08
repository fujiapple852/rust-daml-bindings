use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::version_service_client::VersionServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::GetLedgerApiVersionRequest;
use crate::service::common::make_request;
use log::{debug, trace};
use tonic::transport::Channel;

/// Retrieve information about the ledger API version.
pub struct DamlVersionService<'a> {
    channel: Channel,
    ledger_id: &'a str,
    auth_token: Option<&'a str>,
}

impl<'a> DamlVersionService<'a> {
    pub fn new(channel: Channel, ledger_id: &'a str, auth_token: Option<&'a str>) -> Self {
        Self {
            channel,
            ledger_id,
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

    /// Override the ledger id to use for this service.
    pub fn with_ledger_id(self, ledger_id: &'a str) -> Self {
        Self {
            ledger_id,
            ..self
        }
    }

    /// Read the Ledger API version.
    pub async fn get_ledger_api_version(&self) -> DamlResult<String> {
        debug!("get_ledger_api_version");
        let payload = GetLedgerApiVersionRequest {
            ledger_id: self.ledger_id.to_string(),
        };
        trace!("get_ledger_api_version payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let version = self.client().get_ledger_api_version(make_request(payload, self.auth_token.as_deref())?).await?;
        Ok(version.into_inner().version)
    }

    fn client(&self) -> VersionServiceClient<Channel> {
        VersionServiceClient::new(self.channel.clone())
    }
}
