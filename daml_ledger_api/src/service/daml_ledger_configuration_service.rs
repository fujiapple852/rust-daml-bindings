use crate::data::DamlError;
use crate::data::DamlLedgerConfiguration;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::ledger_configuration_service_client::LedgerConfigurationServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{GetLedgerConfigurationRequest, TraceContext};
use crate::util::Required;
use futures::stream::StreamExt;
use futures::Stream;
use std::convert::TryFrom;
use tonic::transport::Channel;
use tonic::Request;

/// Subscribe to configuration changes of a DAML ledger.
pub struct DamlLedgerConfigurationService {
    channel: Channel,
    ledger_id: String,
}

impl DamlLedgerConfigurationService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
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
        let request = Request::new(GetLedgerConfigurationRequest {
            ledger_id: self.ledger_id.clone(),
            trace_context: trace_context.into().map(TraceContext::from),
        });
        let config_stream = self.client().get_ledger_configuration(request).await?.into_inner();
        Ok(config_stream.map(|item| match item {
            Ok(config) => DamlLedgerConfiguration::try_from(config.ledger_configuration.req()?),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    fn client(&self) -> LedgerConfigurationServiceClient<Channel> {
        LedgerConfigurationServiceClient::new(self.channel.clone())
    }
}
