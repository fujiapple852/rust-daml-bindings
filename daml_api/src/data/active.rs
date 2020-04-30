use crate::data::event::DamlCreatedEvent;
use crate::data::{DamlError, DamlResult, DamlTraceContext};
use crate::grpc_protobuf::com::daml::ledger::api::v1::GetActiveContractsResponse;
use std::convert::TryFrom;

/// A set of active contracts on a DAML ledger.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlActiveContracts {
    pub offset: String,
    pub workflow_id: String,
    pub active_contracts: Vec<DamlCreatedEvent>,
    pub trace_context: Option<DamlTraceContext>,
}

impl DamlActiveContracts {
    pub fn new(
        offset: impl Into<String>,
        workflow_id: impl Into<String>,
        active_contracts: Vec<DamlCreatedEvent>,
        trace_context: Option<DamlTraceContext>,
    ) -> Self {
        Self {
            offset: offset.into(),
            workflow_id: workflow_id.into(),
            active_contracts,
            trace_context,
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

    pub const fn trace_context(&self) -> &Option<DamlTraceContext> {
        &self.trace_context
    }
}

impl TryFrom<GetActiveContractsResponse> for DamlActiveContracts {
    type Error = DamlError;

    fn try_from(active: GetActiveContractsResponse) -> Result<Self, Self::Error> {
        Ok(Self::new(
            active.offset,
            active.workflow_id,
            active.active_contracts.into_iter().map(DamlCreatedEvent::try_from).collect::<DamlResult<Vec<_>>>()?,
            active.trace_context.map(DamlTraceContext::from),
        ))
    }
}
