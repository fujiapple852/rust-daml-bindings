use std::convert::TryFrom;
use std::fmt::Debug;

use futures::Stream;
use futures::StreamExt;
use tonic::transport::Channel;
use tracing::{instrument, trace};

use crate::data::filter::DamlTransactionFilter;
use crate::data::DamlActiveContracts;
use crate::data::DamlError;
use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::active_contracts_service_client::ActiveContractsServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{GetActiveContractsRequest, TransactionFilter};
use crate::service::common::make_request;
use crate::service::DamlVerbosity;

/// Returns a stream of the active contracts on a Daml ledger.
///
/// Allows clients to initialize themselves according to a fairly recent state of the ledger without reading through
/// all transactions that were committed since the ledger’s creation.
///
/// Getting an empty stream means that the active contracts set is empty and the client should listen to transactions
/// using [`DamlLedgerOffsetBoundary::Begin`].  Clients SHOULD NOT assume that the set of active contracts they receive
/// reflects the state at the ledger end.
///
/// [`DamlLedgerOffsetBoundary::Begin`]: crate::data::offset::DamlLedgerOffsetBoundary::Begin
#[derive(Debug)]
pub struct DamlActiveContractsService<'a> {
    channel: Channel,
    ledger_id: &'a str,
    auth_token: Option<&'a str>,
}

impl<'a> DamlActiveContractsService<'a> {
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

    #[instrument(skip(self))]
    pub async fn get_active_contracts(
        &self,
        filter: impl Into<DamlTransactionFilter> + Debug,
        verbose: impl Into<DamlVerbosity> + Debug,
    ) -> DamlResult<impl Stream<Item = DamlResult<DamlActiveContracts>>> {
        let payload = GetActiveContractsRequest {
            ledger_id: self.ledger_id.to_string(),
            filter: Some(TransactionFilter::from(filter.into())),
            verbose: bool::from(verbose.into()),
        };
        trace!(payload = ?payload, token = ?self.auth_token);
        let active_contract_stream =
            self.client().get_active_contracts(make_request(payload, self.auth_token)?).await?.into_inner();
        Ok(active_contract_stream.inspect(|response| trace!(?response)).map(|response| match response {
            Ok(c) => DamlActiveContracts::try_from(c),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    fn client(&self) -> ActiveContractsServiceClient<Channel> {
        ActiveContractsServiceClient::new(self.channel.clone())
    }
}
