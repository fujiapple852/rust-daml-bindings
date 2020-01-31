use crate::data::filter::DamlTransactionFilter;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::{DamlActiveContracts, DamlTraceContext};
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::active_contracts_service_client::ActiveContractsServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{
    GetActiveContractsRequest, TraceContext, TransactionFilter,
};
use crate::service::DamlVerbosity;

use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use futures::Stream;
use futures::StreamExt;
use log::{debug, trace};
use std::convert::TryFrom;
use tonic::transport::Channel;

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
    channel: Channel,
    ledger_id: String,
    auth_token: Option<String>,
}

impl DamlActiveContractsService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
            auth_token,
        }
    }

    pub async fn get_active_contracts(
        &self,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
    ) -> DamlResult<impl Stream<Item = DamlResult<DamlActiveContracts>>> {
        self.get_active_contracts_with_trace(filter, verbose, None).await
    }

    pub async fn get_active_contracts_with_trace(
        &self,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Stream<Item = DamlResult<DamlActiveContracts>>> {
        debug!("get_active_contracts");
        let payload = GetActiveContractsRequest {
            ledger_id: self.ledger_id.clone(),
            filter: Some(TransactionFilter::from(filter.into())),
            verbose: bool::from(verbose.into()),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!("get_active_contracts payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let active_contract_stream =
            self.client().get_active_contracts(make_request(payload, &self.auth_token)?).await?.into_inner();
        Ok(active_contract_stream.map(|c| match c {
            Ok(c) => DamlActiveContracts::try_from(c),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    fn client(&self) -> ActiveContractsServiceClient<Channel> {
        ActiveContractsServiceClient::new(self.channel.clone())
    }
}

impl DamlTokenRefresh for DamlActiveContractsService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
