use grpcio::Channel;

use crate::data::filter::DamlTransactionFilter;
use crate::data::DamlActiveContracts;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::grpc_protobuf_autogen::active_contracts_service::GetActiveContractsRequest;
use crate::grpc_protobuf_autogen::active_contracts_service::GetActiveContractsResponse;
use crate::grpc_protobuf_autogen::active_contracts_service_grpc::ActiveContractsServiceClient;
use crate::service::DamlVerbosity;
use futures::future::{err, ok};
use futures::Stream;
use grpcio::ClientSStreamReceiver;
use std::convert::TryInto;

/// Returns a stream of the active contracts on a DAML ledger.
///
/// Allows clients to initialize themselves according to a fairly recent state of the ledger without reading through
/// all transactions that were committed since the ledger’s creation.
///
/// Getting an empty stream means that the active contracts set is empty and the client should listen to transactions
/// using [`DamlLedgerOffsetBoundary::Begin`].  Clients SHOULD NOT assume that the set of active contracts they receive
/// reflects the state at the ledger end.
///
/// [`DamlLedgerOffsetBoundary::Begin`]: crate::data::offset::DamlLedgerOffsetBoundary::Begin
pub struct DamlActiveContractsService {
    grpc_client: ActiveContractsServiceClient,
    ledger_id: String,
}

impl DamlActiveContractsService {
    pub fn new(channel: Channel, ledger_id: String) -> Self {
        Self {
            grpc_client: ActiveContractsServiceClient::new(channel),
            ledger_id,
        }
    }

    pub fn get_active_contracts(
        &self,
        filter: impl Into<DamlTransactionFilter>,
        verbose: DamlVerbosity,
    ) -> DamlResult<impl Stream<Item = DamlActiveContracts, Error = DamlError>> {
        self.get_active_contracts_with_trace(filter, verbose, None)
    }

    pub fn get_active_contracts_with_trace(
        &self,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Stream<Item = DamlActiveContracts, Error = DamlError>> {
        let mut request = GetActiveContractsRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        request.set_filter(filter.into().into());
        match verbose.into() {
            DamlVerbosity::Verbose => request.set_verbose(true),
            DamlVerbosity::NotVerbose => request.set_verbose(false),
            _ => {},
        }
        if let Some(tc) = trace_context.into() {
            request.set_trace_context(tc.into());
        }
        let async_response: ClientSStreamReceiver<GetActiveContractsResponse> =
            self.grpc_client.get_active_contracts(&request)?;
        Ok(async_response.map_err(Into::into).map(TryInto::try_into).and_then(
            |active_contracts| match active_contracts {
                Ok(active) => ok(active),
                Err(e) => err(e),
            },
        ))
    }
}
