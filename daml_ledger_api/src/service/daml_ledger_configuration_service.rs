use grpcio::Channel;

use crate::data::DamlError;
use crate::data::DamlLedgerConfiguration;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf_autogen::ledger_configuration_service::GetLedgerConfigurationRequest;
use crate::grpc_protobuf_autogen::ledger_configuration_service::GetLedgerConfigurationResponse;
use crate::grpc_protobuf_autogen::ledger_configuration_service_grpc::LedgerConfigurationServiceClient;
use futures::Stream;
use grpcio::ClientSStreamReceiver;

/// Subscribe to configuration changes of a DAML ledger.
pub struct DamlLedgerConfigurationService {
    grpc_client: LedgerConfigurationServiceClient,
    ledger_id: String,
}

impl DamlLedgerConfigurationService {
    pub fn new(channel: Channel, ledger_id: String) -> Self {
        Self {
            grpc_client: LedgerConfigurationServiceClient::new(channel),
            ledger_id,
        }
    }

    /// TODO fully document this
    pub fn get_ledger_configuration(
        &self,
    ) -> DamlResult<impl Stream<Item = DamlLedgerConfiguration, Error = DamlError>> {
        self.get_ledger_configuration_with_trace(None)
    }

    pub fn get_ledger_configuration_with_trace(
        &self,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Stream<Item = DamlLedgerConfiguration, Error = DamlError>> {
        let mut request = GetLedgerConfigurationRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        if let Some(tc) = trace_context.into() {
            request.set_trace_context(tc.into());
        }
        let async_response: ClientSStreamReceiver<GetLedgerConfigurationResponse> =
            self.grpc_client.get_ledger_configuration(&request)?;
        Ok(async_response
            .map_err(Into::into)
            .map(|mut r: GetLedgerConfigurationResponse| r.take_ledger_configuration().into()))
    }
}
