use crate::data::DamlError;
use crate::data::DamlLedgerConfiguration;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf::com::daml::ledger::api::v1::ledger_configuration_service_client::LedgerConfigurationServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{GetLedgerConfigurationRequest, TraceContext};
use crate::service::common::make_request;
use crate::util::Required;
use futures::stream::StreamExt;
use futures::Stream;
use std::convert::TryFrom;
use tonic::transport::Channel;
use tracing::{instrument, trace};

/// Subscribe to configuration changes of a DAML ledger.
#[derive(Debug)]
pub struct DamlLedgerConfigurationService<'a> {
    channel: Channel,
    ledger_id: &'a str,
    auth_token: Option<&'a str>,
}

impl<'a> DamlLedgerConfigurationService<'a> {
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

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_ledger_configuration(
        &self,
    ) -> DamlResult<impl Stream<Item = DamlResult<DamlLedgerConfiguration>>> {
        self.get_ledger_configuration_with_trace(None).await
    }

    pub async fn get_ledger_configuration_with_trace(
        &self,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Stream<Item = DamlResult<DamlLedgerConfiguration>>> {
        let payload = GetLedgerConfigurationRequest {
            ledger_id: self.ledger_id.to_string(),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let config_stream = self
            .client()
            .get_ledger_configuration(make_request(payload, self.auth_token)?)
            .await?
            .into_inner();
        Ok(config_stream.inspect(|response| trace!(?response)).map(|item| match item {
            Ok(config) => DamlLedgerConfiguration::try_from(config.ledger_configuration.req()?),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    fn client(&self) -> LedgerConfigurationServiceClient<Channel> {
        LedgerConfigurationServiceClient::new(self.channel.clone())
    }
}
