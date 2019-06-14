use futures::stream::Stream;
use grpcio::Channel;
use grpcio::ClientSStreamReceiver;

use crate::data::filter::DamlTransactionFilter;
use crate::data::offset::{DamlLedgerOffset, DamlLedgerOffsetType};
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTraceContext;
use crate::data::DamlTransaction;
use crate::data::DamlTransactionTree;
use crate::grpc_protobuf_autogen::transaction::Transaction;
use crate::grpc_protobuf_autogen::transaction::TransactionTree;
use crate::grpc_protobuf_autogen::transaction_service::GetLedgerEndResponse;
use crate::grpc_protobuf_autogen::transaction_service::GetTransactionByEventIdRequest;
use crate::grpc_protobuf_autogen::transaction_service::GetTransactionByIdRequest;
use crate::grpc_protobuf_autogen::transaction_service::GetTransactionResponse;
use crate::grpc_protobuf_autogen::transaction_service::GetTransactionTreesResponse;
use crate::grpc_protobuf_autogen::transaction_service::GetTransactionsRequest;
use crate::grpc_protobuf_autogen::transaction_service::GetTransactionsResponse;
use crate::grpc_protobuf_autogen::transaction_service::{GetFlatTransactionResponse, GetLedgerEndRequest};
use crate::grpc_protobuf_autogen::transaction_service_grpc::TransactionServiceClient;
use crate::service::DamlVerbosity;
use futures::future::{err, ok};
use futures::Future;
use grpcio::ClientUnaryReceiver;
use protobuf::RepeatedField;
use std::convert::TryInto;

/// Read transactions from a DAML ledger.
pub struct DamlTransactionService {
    grpc_client: TransactionServiceClient,
    ledger_id: String,
}

impl DamlTransactionService {
    pub fn new(channel: Channel, ledger_id: String) -> Self {
        Self {
            grpc_client: TransactionServiceClient::new(channel),
            ledger_id,
        }
    }

    /// TODO fully document this
    pub fn get_transactions(
        &self,
        begin: impl Into<DamlLedgerOffset>,
        end: impl Into<DamlLedgerOffsetType>,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
    ) -> DamlResult<impl Stream<Item = Vec<DamlTransaction>, Error = DamlError>> {
        self.get_transactions_with_trace(begin, end, filter, verbose, None)
    }

    pub fn get_transactions_with_trace(
        &self,
        begin: impl Into<DamlLedgerOffset>,
        end: impl Into<DamlLedgerOffsetType>,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Stream<Item = Vec<DamlTransaction>, Error = DamlError>> {
        let mut request = GetTransactionsRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        request.set_begin(begin.into().into());
        match end.into() {
            DamlLedgerOffsetType::Unbounded => {},
            DamlLedgerOffsetType::Bounded(b) => request.set_end(b.into()),
        };
        request.set_filter(filter.into().into());
        match verbose.into() {
            DamlVerbosity::Verbose => request.set_verbose(true),
            DamlVerbosity::NotVerbose => request.set_verbose(false),
            _ => {},
        }
        if let Some(tc) = trace_context.into() {
            request.set_trace_context(tc.into());
        }
        let async_response: ClientSStreamReceiver<GetTransactionsResponse> =
            self.grpc_client.get_transactions(&request)?;
        Ok(async_response
            .map_err(Into::into)
            .map(|mut r: GetTransactionsResponse| {
                (r.take_transactions() as RepeatedField<Transaction>)
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<DamlResult<Vec<DamlTransaction>>>()
            })
            .and_then(|transactions| match transactions {
                Ok(txs) => ok(txs),
                Err(e) => err(e),
            }))
    }

    /// TODO fully document this
    pub fn get_transaction_trees(
        &self,
        begin: impl Into<DamlLedgerOffset>,
        end: impl Into<DamlLedgerOffsetType>,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
    ) -> DamlResult<impl Stream<Item = Vec<DamlTransactionTree>, Error = DamlError>> {
        self.get_transaction_trees_with_trace(begin, end, filter, verbose, None)
    }

    pub fn get_transaction_trees_with_trace(
        &self,
        begin: impl Into<DamlLedgerOffset>,
        end: impl Into<DamlLedgerOffsetType>,
        filter: impl Into<DamlTransactionFilter>,
        verbose: impl Into<DamlVerbosity>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> DamlResult<impl Stream<Item = Vec<DamlTransactionTree>, Error = DamlError>> {
        let mut request = GetTransactionsRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        request.set_begin(begin.into().into());
        match end.into() {
            DamlLedgerOffsetType::Unbounded => {},
            DamlLedgerOffsetType::Bounded(b) => request.set_end(b.into()),
        };
        request.set_filter(filter.into().into());
        match verbose.into() {
            DamlVerbosity::Verbose => request.set_verbose(true),
            DamlVerbosity::NotVerbose => request.set_verbose(false),
            _ => {},
        }
        if let Some(tc) = trace_context.into() {
            request.set_trace_context(tc.into());
        }
        let async_response: ClientSStreamReceiver<GetTransactionTreesResponse> =
            self.grpc_client.get_transaction_trees(&request)?;
        Ok(async_response
            .map_err(Into::into)
            .map(|mut r: GetTransactionTreesResponse| {
                (r.take_transactions() as RepeatedField<TransactionTree>)
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<DamlResult<Vec<DamlTransactionTree>>>()
            })
            .and_then(|transactions| match transactions {
                Ok(txs) => ok(txs),
                Err(e) => err(e),
            }))
    }

    /// TODO fully document this
    pub fn get_transaction_by_event_id(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<impl Future<Item = DamlTransactionTree, Error = DamlError>> {
        self.get_transaction_by_event_id_with_trace(event_id, parties, None)
    }

    pub fn get_transaction_by_event_id_sync(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<DamlTransactionTree> {
        self.get_transaction_by_event_id_with_trace(event_id, parties, None)?.wait()
    }

    pub fn get_transaction_by_event_id_with_trace_sync(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<DamlTransactionTree> {
        self.get_transaction_by_event_id_with_trace(event_id, parties, trace_context)?.wait()
    }

    pub fn get_transaction_by_event_id_with_trace(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<impl Future<Item = DamlTransactionTree, Error = DamlError>> {
        let request = self.make_by_event_id_request(event_id, parties, trace_context);
        let async_response: ClientUnaryReceiver<GetTransactionResponse> =
            self.grpc_client.get_transaction_by_event_id_async(&request)?;
        Self::process_transaction_response(async_response)
    }

    /// TODO fully document this
    pub fn get_transaction_by_id(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<impl Future<Item = DamlTransactionTree, Error = DamlError>> {
        self.get_transaction_by_id_with_trace(transaction_id, parties, None)
    }

    pub fn get_transaction_by_id_sync(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<DamlTransactionTree> {
        self.get_transaction_by_id_with_trace(transaction_id, parties, None)?.wait()
    }

    pub fn get_transaction_by_id_with_trace_sync(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<DamlTransactionTree> {
        self.get_transaction_by_id_with_trace(transaction_id, parties, trace_context)?.wait()
    }

    pub fn get_transaction_by_id_with_trace(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<impl Future<Item = DamlTransactionTree, Error = DamlError>> {
        let request = self.make_by_id_request(transaction_id, parties, trace_context);
        let async_response: ClientUnaryReceiver<GetTransactionResponse> =
            self.grpc_client.get_transaction_by_id_async(&request)?;
        Self::process_transaction_response(async_response)
    }

    /// TODO fully document this
    pub fn get_flat_transaction_by_event_id(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<impl Future<Item = DamlTransaction, Error = DamlError>> {
        self.get_flat_transaction_by_event_id_with_trace(event_id, parties, None)
    }

    pub fn get_flat_transaction_by_event_id_sync(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<DamlTransaction> {
        self.get_flat_transaction_by_event_id_with_trace(event_id, parties, None)?.wait()
    }

    pub fn get_flat_transaction_by_event_id_with_trace_sync(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<DamlTransaction> {
        self.get_flat_transaction_by_event_id_with_trace(event_id, parties, trace_context)?.wait()
    }

    pub fn get_flat_transaction_by_event_id_with_trace(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<impl Future<Item = DamlTransaction, Error = DamlError>> {
        let request = self.make_by_event_id_request(event_id, parties, trace_context);
        let async_response: ClientUnaryReceiver<GetFlatTransactionResponse> =
            self.grpc_client.get_flat_transaction_by_event_id_async(&request)?;
        Self::process_flat_transaction_response(async_response)
    }

    /// TODO fully document this
    pub fn get_flat_transaction_by_id(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<impl Future<Item = DamlTransaction, Error = DamlError>> {
        self.get_flat_transaction_by_id_with_trace(transaction_id, parties, None)
    }

    pub fn get_flat_transaction_by_id_sync(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> DamlResult<DamlTransaction> {
        self.get_flat_transaction_by_id_with_trace(transaction_id, parties, None)?.wait()
    }

    pub fn get_flat_transaction_by_id_with_trace_sync(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<DamlTransaction> {
        self.get_flat_transaction_by_id_with_trace(transaction_id, parties, trace_context)?.wait()
    }

    pub fn get_flat_transaction_by_id_with_trace(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<impl Future<Item = DamlTransaction, Error = DamlError>> {
        let request = self.make_by_id_request(transaction_id, parties, trace_context);
        let async_response: ClientUnaryReceiver<GetFlatTransactionResponse> =
            self.grpc_client.get_flat_transaction_by_id_async(&request)?;
        Self::process_flat_transaction_response(async_response)
    }

    /// TODO fully document this
    pub fn get_ledger_end(&self) -> DamlResult<impl Future<Item = DamlLedgerOffset, Error = DamlError>> {
        self.get_ledger_end_with_trace(None)
    }

    pub fn get_ledger_end_sync(&self) -> DamlResult<DamlLedgerOffset> {
        self.get_ledger_end_with_trace(None)?.wait()
    }

    pub fn get_ledger_end_with_trace_sync(
        &self,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<DamlLedgerOffset> {
        self.get_ledger_end_with_trace(trace_context)?.wait()
    }

    pub fn get_ledger_end_with_trace(
        &self,
        trace_context: Option<DamlTraceContext>,
    ) -> DamlResult<impl Future<Item = DamlLedgerOffset, Error = DamlError>> {
        let mut request = GetLedgerEndRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        if let Some(tc) = trace_context {
            request.set_trace_context(tc.into());
        }
        let async_response: ClientUnaryReceiver<GetLedgerEndResponse> =
            self.grpc_client.get_ledger_end_async(&request)?;
        Ok(async_response.map_err(Into::into).map(|mut r| r.take_offset().try_into()).and_then(|end| match end {
            Ok(le) => ok(le),
            Err(e) => err(e),
        }))
    }

    fn make_by_event_id_request(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> GetTransactionByEventIdRequest {
        let mut request = GetTransactionByEventIdRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        request.set_event_id(event_id.into());
        request.set_requesting_parties(parties.into().into());
        if let Some(tc) = trace_context {
            request.set_trace_context(tc.into());
        }
        request
    }

    fn make_by_id_request(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
        trace_context: Option<DamlTraceContext>,
    ) -> GetTransactionByIdRequest {
        let mut request = GetTransactionByIdRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        request.set_transaction_id(transaction_id.into());
        request.set_requesting_parties(parties.into().into());
        if let Some(tc) = trace_context {
            request.set_trace_context(tc.into());
        }
        request
    }

    fn process_transaction_response(
        async_response: ClientUnaryReceiver<GetTransactionResponse>,
    ) -> DamlResult<impl Future<Item = DamlTransactionTree, Error = DamlError>> {
        Ok(async_response.map_err(Into::into).map(|mut r| r.take_transaction().try_into()).and_then(|transaction| {
            match transaction {
                Ok(tx) => ok(tx),
                Err(e) => err(e),
            }
        }))
    }

    fn process_flat_transaction_response(
        async_response: ClientUnaryReceiver<GetFlatTransactionResponse>,
    ) -> DamlResult<impl Future<Item = DamlTransaction, Error = DamlError>> {
        Ok(async_response.map_err(Into::into).map(|mut r| r.take_transaction().try_into()).and_then(|transaction| {
            match transaction {
                Ok(tx) => ok(tx),
                Err(e) => err(e),
            }
        }))
    }
}
