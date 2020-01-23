use crate::data::filter::DamlTransactionFilter;
use crate::data::offset::{DamlLedgerOffset, DamlLedgerOffsetType};
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::data::DamlTransaction;
use crate::data::DamlTransactionTree;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::transaction_service_client::TransactionServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{
    GetLedgerEndRequest, GetTransactionByEventIdRequest, GetTransactionByIdRequest, GetTransactionsRequest,
    LedgerOffset, TraceContext, TransactionFilter,
};
use crate::service::DamlVerbosity;
use crate::util::Required;

use futures::Stream;
use futures::StreamExt;
use std::convert::TryFrom;
use tonic::transport::Channel;
use tonic::Request;

/// Read transactions from a DAML ledger.
pub struct DamlTransactionService {
    channel: Channel,
    ledger_id: String,
}

impl DamlTransactionService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
        }
    }

    /// DOCME fully document this
    pub async fn get_transactions(
        &self,
        begin: impl Into<DamlLedgerOffset>,
        end: impl Into<DamlLedgerOffsetType>,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
    ) -> DamlResult<impl Stream<Item = DamlResult<Vec<DamlTransaction>>>> {
        self.get_transactions_with_trace(begin, end, filter, verbose, None).await
    }

    /// DOCME fully document this
    pub async fn get_transactions_with_trace(
        &self,
        begin: impl Into<DamlLedgerOffset>,
        end: impl Into<DamlLedgerOffsetType>,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Stream<Item = DamlResult<Vec<DamlTransaction>>>> {
        let request = self.make_transactions_request(begin, end, filter, verbose, trace_context);
        let transaction_stream = self.client().get_transactions(request).await?.into_inner();
        Ok(transaction_stream.map(|item| match item {
            Ok(r) => Ok(r
                .transactions
                .into_iter()
                .map(DamlTransaction::try_from)
                .collect::<DamlResult<Vec<DamlTransaction>>>()?),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    /// DOCME fully document this
    pub async fn get_transaction_trees(
        &self,
        begin: impl Into<DamlLedgerOffset>,
        end: impl Into<DamlLedgerOffsetType>,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
    ) -> DamlResult<impl Stream<Item = DamlResult<Vec<DamlTransactionTree>>>> {
        self.get_transaction_trees_with_trace(begin, end, filter, verbose, None).await
    }

    /// DOCME fully document this
    pub async fn get_transaction_trees_with_trace(
        &self,
        begin: impl Into<DamlLedgerOffset>,
        end: impl Into<DamlLedgerOffsetType>,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Stream<Item = DamlResult<Vec<DamlTransactionTree>>>> {
        let request = self.make_transactions_request(begin, end, filter, verbose, trace_context);
        let transaction_stream = self.client().get_transaction_trees(request).await?.into_inner();
        Ok(transaction_stream.map(|item| match item {
            Ok(r) => Ok(r
                .transactions
                .into_iter()
                .map(DamlTransactionTree::try_from)
                .collect::<DamlResult<Vec<DamlTransactionTree>>>()?),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    /// DOCME fully document this
    pub async fn get_transaction_by_event_id(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<DamlTransactionTree> {
        self.get_transaction_by_event_id_with_trace(event_id, parties, None).await
    }

    /// DOCME fully document this
    pub async fn get_transaction_by_event_id_with_trace(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<DamlTransactionTree> {
        let request = self.make_by_event_id_request(event_id, parties, trace_context);
        let response = self.client().get_transaction_by_event_id(request).await?.into_inner();
        DamlTransactionTree::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    pub async fn get_transaction_by_id(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<DamlTransactionTree> {
        self.get_transaction_by_id_with_trace(transaction_id, parties, None).await
    }

    /// DOCME fully document this
    pub async fn get_transaction_by_id_with_trace(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<DamlTransactionTree> {
        let request = self.make_by_id_request(transaction_id, parties, trace_context);
        let response = self.client().get_transaction_by_id(request).await?.into_inner();
        DamlTransactionTree::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    pub async fn get_flat_transaction_by_event_id(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<DamlTransaction> {
        self.get_flat_transaction_by_event_id_with_trace(event_id, parties, None).await
    }

    /// DOCME fully document this
    pub async fn get_flat_transaction_by_event_id_with_trace(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<DamlTransaction> {
        let request = self.make_by_event_id_request(event_id, parties, trace_context);
        let response = self.client().get_flat_transaction_by_event_id(request).await?.into_inner();
        DamlTransaction::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    pub async fn get_flat_transaction_by_id(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<DamlTransaction> {
        self.get_flat_transaction_by_id_with_trace(transaction_id, parties, None).await
    }

    /// DOCME fully document this
    pub async fn get_flat_transaction_by_id_with_trace(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<DamlTransaction> {
        let request = self.make_by_id_request(transaction_id, parties, trace_context);
        let response = self.client().get_flat_transaction_by_id(request).await?.into_inner();
        DamlTransaction::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    pub async fn get_ledger_end(&self) -> DamlResult<DamlLedgerOffset> {
        self.get_ledger_end_with_trace(None).await
    }

    /// DOCME fully document this
    pub async fn get_ledger_end_with_trace(
        &self,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<DamlLedgerOffset> {
        let request = Request::new(GetLedgerEndRequest {
            ledger_id: self.ledger_id.clone(),
            trace_context: trace_context.map(TraceContext::from),
        });
        let response = self.client().get_ledger_end(request).await?.into_inner();
        DamlLedgerOffset::try_from(response.offset.req()?)
    }

    fn client(&self) -> TransactionServiceClient<Channel> {
        TransactionServiceClient::new(self.channel.clone())
    }

    fn make_transactions_request(
        &self,
        begin: impl Into<DamlLedgerOffset>,
        end: impl Into<DamlLedgerOffsetType>,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> Request<GetTransactionsRequest> {
        Request::new(GetTransactionsRequest {
            ledger_id: self.ledger_id.clone(),
            begin: Some(LedgerOffset::from(begin.into())),
            end: match end.into() {
                DamlLedgerOffsetType::Unbounded => None,
                DamlLedgerOffsetType::Bounded(b) => Some(LedgerOffset::from(b)),
            },
            filter: Some(TransactionFilter::from(filter.into())),
            verbose: bool::from(verbose.into()),
            trace_context: trace_context.into().map(TraceContext::from),
        })
    }

    fn make_by_event_id_request(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> Request<GetTransactionByEventIdRequest> {
        Request::new(GetTransactionByEventIdRequest {
            ledger_id: self.ledger_id.clone(),
            event_id: event_id.into(),
            requesting_parties: parties.into(),
            trace_context: trace_context.map(TraceContext::from),
        })
    }

    fn make_by_id_request(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> Request<GetTransactionByIdRequest> {
        Request::new(GetTransactionByIdRequest {
            ledger_id: self.ledger_id.clone(),
            transaction_id: transaction_id.into(),
            requesting_parties: parties.into(),
            trace_context: trace_context.map(TraceContext::from),
        })
    }
}
