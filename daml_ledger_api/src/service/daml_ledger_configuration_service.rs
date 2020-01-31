use crate::data::DamlError;
use crate::data::DamlLedgerConfiguration;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::ledger_configuration_service_client::LedgerConfigurationServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{GetLedgerConfigurationRequest, TraceContext};
use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use crate::util::Required;
use futures::stream::StreamExt;
use futures::Stream;
use log::{debug, trace};
use std::convert::TryFrom;
use tonic::transport::Channel;

/// Subscribe to configuration changes of a DAML ledger.
pub struct DamlLedgerConfigurationService {
    channel: Channel,
    ledger_id: String,
    auth_token: Option<String>,
}

impl DamlLedgerConfigurationService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
            auth_token,
        }
    }

    /// DOCME fully document this
    pub async fn get_ledger_configuration(
        &self,
    ) -> DamlResult<impl Stream<Item = DamlResult<DamlLedgerConfiguration>>> {
        self.get_ledger_configuration_with_trace(None).await
    }

    pub async fn get_ledger_configuration_with_trace(
        &self,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Stream<Item = DamlResult<DamlLedgerConfiguration>>> {
        debug!("get_ledger_configuration");
        let payload = GetLedgerConfigurationRequest {
            ledger_id: self.ledger_id.clone(),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("get_ledger_configuration payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let config_stream =
            self.client().get_ledger_configuration(make_request(payload, &self.auth_token)?).await?.into_inner();
        Ok(config_stream.map(|item| match item {
            Ok(config) => DamlLedgerConfiguration::try_from(config.ledger_configuration.req()?),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    fn client(&self) -> LedgerConfigurationServiceClient<Channel> {
        LedgerConfigurationServiceClient::new(self.channel.clone())
    }
}

impl DamlTokenRefresh for DamlLedgerConfigurationService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
