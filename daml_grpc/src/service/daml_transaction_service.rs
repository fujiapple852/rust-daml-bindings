use crate::data::filter::DamlTransactionFilter;
use crate::data::offset::{DamlLedgerOffset, DamlLedgerOffsetType};
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::data::DamlTransaction;
use crate::data::DamlTransactionTree;
use crate::grpc_protobuf::com::daml::ledger::api::v1::transaction_service_client::TransactionServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{
    GetLedgerEndRequest, GetTransactionByEventIdRequest, GetTransactionByIdRequest, GetTransactionTreesResponse,
    GetTransactionsRequest, GetTransactionsResponse, LedgerOffset, TraceContext, TransactionFilter,
};
use crate::service::common::make_request;
use crate::service::DamlVerbosity;
use crate::util::Required;
use futures::Stream;
use futures::StreamExt;
use std::convert::TryFrom;
use std::fmt::Debug;
use tonic::transport::Channel;
use tracing::{instrument, trace};

/// Read transactions from a DAML ledger.
#[derive(Debug)]
pub struct DamlTransactionService<'a> {
    channel: Channel,
    ledger_id: &'a str,
    auth_token: Option<&'a str>,
}

impl<'a> DamlTransactionService<'a> {
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
    #[instrument(skip(self, begin, end, filter, verbose))]
    pub async fn get_transactions(
        &self,
        begin: impl Into<DamlLedgerOffset> + Debug,
        end: impl Into<DamlLedgerOffsetType> + Debug,
        filter: impl Into<DamlTransactionFilter> + Debug,
        verbose: impl Into<DamlVerbosity> + Debug,
    ) -> DamlResult<impl Stream<Item = DamlResult<Vec<DamlTransaction>>>> {
        self.get_transactions_with_trace(begin, end, filter, verbose, None).await
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_transactions_with_trace(
        &self,
        begin: impl Into<DamlLedgerOffset> + Debug,
        end: impl Into<DamlLedgerOffsetType> + Debug,
        filter: impl Into<DamlTransactionFilter> + Debug,
        verbose: impl Into<DamlVerbosity> + Debug,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<impl Stream<Item = DamlResult<Vec<DamlTransaction>>>> {
        let payload = self.make_transactions_payload(begin, end, filter, verbose, trace_context);
        trace!(payload = ?payload, token = ?self.auth_token);
        let transaction_stream =
            self.client().get_transactions(make_request(payload, self.auth_token)?).await?.into_inner();
        Ok(transaction_stream.inspect(|response| trace!(?response)).map(|item: Result<GetTransactionsResponse, _>| {
            match item {
                Ok(r) => Ok(r
                    .transactions
                    .into_iter()
                    .map(DamlTransaction::try_from)
                    .collect::<DamlResult<Vec<DamlTransaction>>>()?),
                Err(e) => Err(DamlError::from(e)),
            }
        }))
    }

    /// DOCME fully document this
    #[instrument(skip(self, begin, end, filter, verbose))]
    pub async fn get_transaction_trees(
        &self,
        begin: impl Into<DamlLedgerOffset> + Debug,
        end: impl Into<DamlLedgerOffsetType> + Debug,
        filter: impl Into<DamlTransactionFilter> + Debug,
        verbose: impl Into<DamlVerbosity> + Debug,
    ) -> DamlResult<impl Stream<Item = DamlResult<Vec<DamlTransactionTree>>>> {
        self.get_transaction_trees_with_trace(begin, end, filter, verbose, None).await
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_transaction_trees_with_trace(
        &self,
        begin: impl Into<DamlLedgerOffset> + Debug,
        end: impl Into<DamlLedgerOffsetType> + Debug,
        filter: impl Into<DamlTransactionFilter> + Debug,
        verbose: impl Into<DamlVerbosity> + Debug,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<impl Stream<Item = DamlResult<Vec<DamlTransactionTree>>>> {
        let payload = self.make_transactions_payload(begin, end, filter, verbose, trace_context);
        trace!(payload = ?payload, token = ?self.auth_token);
        let transaction_stream =
            self.client().get_transaction_trees(make_request(payload, self.auth_token)?).await?.into_inner();
        Ok(transaction_stream.inspect(|response| trace!(?response)).map(
            |item: Result<GetTransactionTreesResponse, _>| match item {
                Ok(r) => Ok(r
                    .transactions
                    .into_iter()
                    .map(DamlTransactionTree::try_from)
                    .collect::<DamlResult<Vec<DamlTransactionTree>>>()?),
                Err(e) => Err(DamlError::from(e)),
            },
        ))
    }

    /// DOCME fully document this
    #[instrument(skip(self, event_id, parties))]
    pub async fn get_transaction_by_event_id(
        &self,
        event_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
    ) -> DamlResult<DamlTransactionTree> {
        self.get_transaction_by_event_id_with_trace(event_id, parties, None).await
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_transaction_by_event_id_with_trace(
        &self,
        event_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<DamlTransactionTree> {
        let payload = self.make_by_event_id_payload(event_id, parties, trace_context.into());
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().get_transaction_by_event_id(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        DamlTransactionTree::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    #[instrument(skip(self, transaction_id, parties))]
    pub async fn get_transaction_by_id(
        &self,
        transaction_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
    ) -> DamlResult<DamlTransactionTree> {
        self.get_transaction_by_id_with_trace(transaction_id, parties, None).await
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_transaction_by_id_with_trace(
        &self,
        transaction_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<DamlTransactionTree> {
        let payload = self.make_by_id_payload(transaction_id, parties, trace_context.into());
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self.client().get_transaction_by_id(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        DamlTransactionTree::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    #[instrument(skip(self, event_id, parties))]
    pub async fn get_flat_transaction_by_event_id(
        &self,
        event_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
    ) -> DamlResult<DamlTransaction> {
        self.get_flat_transaction_by_event_id_with_trace(event_id, parties, None).await
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_flat_transaction_by_event_id_with_trace(
        &self,
        event_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<DamlTransaction> {
        let payload = self.make_by_event_id_payload(event_id, parties, trace_context.into());
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().get_flat_transaction_by_event_id(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        DamlTransaction::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    #[instrument(skip(self, transaction_id, parties))]
    pub async fn get_flat_transaction_by_id(
        &self,
        transaction_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
    ) -> DamlResult<DamlTransaction> {
        self.get_flat_transaction_by_id_with_trace(transaction_id, parties, None).await
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_flat_transaction_by_id_with_trace(
        &self,
        transaction_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<DamlTransaction> {
        let payload = self.make_by_id_payload(transaction_id, parties, trace_context.into());
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().get_flat_transaction_by_id(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        DamlTransaction::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_ledger_end(&self) -> DamlResult<DamlLedgerOffset> {
        self.get_ledger_end_with_trace(None).await
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_ledger_end_with_trace(
        &self,
        trace_context: impl Into<Option<DamlTraceContext>> + Debug,
    ) -> DamlResult<DamlLedgerOffset> {
        let payload = GetLedgerEndRequest {
            ledger_id: self.ledger_id.to_string(),
            trace_context: trace_context.into().map(TraceContext::from),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self.client().get_ledger_end(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        DamlLedgerOffset::try_from(response.offset.req()?)
    }

    fn client(&self) -> TransactionServiceClient<Channel> {
        TransactionServiceClient::new(self.channel.clone())
    }

    fn make_transactions_payload(
        &self,
        begin: impl Into<DamlLedgerOffset>,
        end: impl Into<DamlLedgerOffsetType>,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> GetTransactionsRequest {
        GetTransactionsRequest {
            ledger_id: self.ledger_id.to_string(),
            begin: Some(LedgerOffset::from(begin.into())),
            end: match end.into() {
                DamlLedgerOffsetType::Unbounded => None,
                DamlLedgerOffsetType::Bounded(b) => Some(LedgerOffset::from(b)),
            },
            filter: Some(TransactionFilter::from(filter.into())),
            verbose: bool::from(verbose.into()),
            trace_context: trace_context.into().map(TraceContext::from),
        }
    }

    fn make_by_event_id_payload(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> GetTransactionByEventIdRequest {
        GetTransactionByEventIdRequest {
            ledger_id: self.ledger_id.to_string(),
            event_id: event_id.into(),
            requesting_parties: parties.into(),
            trace_context: trace_context.map(TraceContext::from),
        }
    }

    fn make_by_id_payload(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> GetTransactionByIdRequest {
        GetTransactionByIdRequest {
            ledger_id: self.ledger_id.to_string(),
            transaction_id: transaction_id.into(),
            requesting_parties: parties.into(),
            trace_context: trace_context.map(TraceContext::from),
        }
    }
}
