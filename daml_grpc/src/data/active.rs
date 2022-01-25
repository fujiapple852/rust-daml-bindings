use crate::data::event::DamlCreatedEvent;
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::GetActiveContractsResponse;
use std::convert::TryFrom;

/// A set of active contracts on a DAML ledger.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlActiveContracts {
    offset: String,
    workflow_id: String,
    active_contracts: Vec<DamlCreatedEvent>,
}

impl DamlActiveContracts {
    pub fn new(
        offset: impl Into<String>,
        workflow_id: impl Into<String>,
        active_contracts: Vec<DamlCreatedEvent>,
    ) -> Self {
        Self {
            offset: offset.into(),
            workflow_id: workflow_id.into(),
            active_contracts,
        }
    }

    pub fn offset(&self) -> &str {
        &self.offset
    }

    pub fn workflow_id(&self) -> &str {
        &self.workflow_id
    }

    pub fn active_contracts(&self) -> &[DamlCreatedEvent] {
        &self.active_contracts
    }
}

impl TryFrom<GetActiveContractsResponse> for DamlActiveContracts {
    type Error = DamlError;

    fn try_from(active: GetActiveContractsResponse) -> Result<Self, Self::Error> {
        Ok(Self::new(
            active.offset,
            active.workflow_id,
            active.active_contracts.into_iter().map(DamlCreatedEvent::try_from).collect::<DamlResult<Vec<_>>>()?,
        ))
    }
}
