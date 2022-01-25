use std::convert::TryFrom;
use std::fmt::Debug;

use futures::Stream;
use futures::StreamExt;
use tonic::transport::Channel;
use tracing::{instrument, trace};

use crate::data::filter::DamlTransactionFilter;
use crate::data::offset::{DamlLedgerOffset, DamlLedgerOffsetType};
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::data::DamlTransaction;
use crate::data::DamlTransactionTree;
use crate::grpc_protobuf::com::daml::ledger::api::v1::transaction_service_client::TransactionServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{
    GetLedgerEndRequest, GetTransactionByEventIdRequest, GetTransactionByIdRequest, GetTransactionTreesResponse,
    GetTransactionsRequest, GetTransactionsResponse, LedgerOffset, TransactionFilter,
};
use crate::service::common::make_request;
use crate::service::DamlVerbosity;
use crate::util::Required;

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
    #[instrument(skip(self))]
    pub async fn get_transactions(
        &self,
        begin: impl Into<DamlLedgerOffset> + Debug,
        end: impl Into<DamlLedgerOffsetType> + Debug,
        filter: impl Into<DamlTransactionFilter> + Debug,
        verbose: impl Into<DamlVerbosity> + Debug,
    ) -> DamlResult<impl Stream<Item = DamlResult<Vec<DamlTransaction>>>> {
        let payload = self.make_transactions_payload(begin, end, filter, verbose);
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
    #[instrument(skip(self))]
    pub async fn get_transaction_trees(
        &self,
        begin: impl Into<DamlLedgerOffset> + Debug,
        end: impl Into<DamlLedgerOffsetType> + Debug,
        filter: impl Into<DamlTransactionFilter> + Debug,
        verbose: impl Into<DamlVerbosity> + Debug,
    ) -> DamlResult<impl Stream<Item = DamlResult<Vec<DamlTransactionTree>>>> {
        let payload = self.make_transactions_payload(begin, end, filter, verbose);
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
    #[instrument(skip(self))]
    pub async fn get_transaction_by_event_id(
        &self,
        event_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
    ) -> DamlResult<DamlTransactionTree> {
        let payload = self.make_by_event_id_payload(event_id, parties);
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().get_transaction_by_event_id(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        DamlTransactionTree::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_transaction_by_id(
        &self,
        transaction_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
    ) -> DamlResult<DamlTransactionTree> {
        let payload = self.make_by_id_payload(transaction_id, parties);
        trace!(payload = ?payload, token = ?self.auth_token);
        let response = self.client().get_transaction_by_id(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        DamlTransactionTree::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_flat_transaction_by_event_id(
        &self,
        event_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
    ) -> DamlResult<DamlTransaction> {
        let payload = self.make_by_event_id_payload(event_id, parties);
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().get_flat_transaction_by_event_id(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        DamlTransaction::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_flat_transaction_by_id(
        &self,
        transaction_id: impl Into<String> + Debug,
        parties: impl Into<Vec<String>> + Debug,
    ) -> DamlResult<DamlTransaction> {
        let payload = self.make_by_id_payload(transaction_id, parties);
        trace!(payload = ?payload, token = ?self.auth_token);
        let response =
            self.client().get_flat_transaction_by_id(make_request(payload, self.auth_token)?).await?.into_inner();
        trace!(?response);
        DamlTransaction::try_from(response.transaction.req()?)
    }

    /// DOCME fully document this
    #[instrument(skip(self))]
    pub async fn get_ledger_end(&self) -> DamlResult<DamlLedgerOffset> {
        let payload = GetLedgerEndRequest {
            ledger_id: self.ledger_id.to_string(),
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
        }
    }

    fn make_by_event_id_payload(
        &self,
        event_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> GetTransactionByEventIdRequest {
        GetTransactionByEventIdRequest {
            ledger_id: self.ledger_id.to_string(),
            event_id: event_id.into(),
            requesting_parties: parties.into(),
        }
    }

    fn make_by_id_payload(
        &self,
        transaction_id: impl Into<String>,
        parties: impl Into<Vec<String>>,
    ) -> GetTransactionByIdRequest {
        GetTransactionByIdRequest {
            ledger_id: self.ledger_id.to_string(),
            transaction_id: transaction_id.into(),
            requesting_parties: parties.into(),
        }
    }
}
