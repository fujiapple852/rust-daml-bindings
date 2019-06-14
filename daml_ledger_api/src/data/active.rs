use crate::data::event::DamlCreatedEvent;
use crate::data::trace::DamlTraceContext;
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf_autogen::active_contracts_service::GetActiveContractsResponse;
use crate::grpc_protobuf_autogen::event::CreatedEvent;
use protobuf::RepeatedField;
use std::convert::{TryFrom, TryInto};

/// A set of active contracts on a DAML ledger.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlActiveContracts {
    offset: String,
    workflow_id: String,
    active_contracts: Vec<DamlCreatedEvent>,
    trace_context: Option<DamlTraceContext>,
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

    pub fn trace_context(&self) -> &Option<DamlTraceContext> {
        &self.trace_context
    }
}

impl TryFrom<GetActiveContractsResponse> for DamlActiveContracts {
    type Error = DamlError;

    fn try_from(mut active: GetActiveContractsResponse) -> Result<Self, Self::Error> {
        let active_contracts = (active.take_active_contracts() as RepeatedField<CreatedEvent>)
            .into_iter()
            .map(TryInto::try_into)
            .collect::<DamlResult<Vec<_>>>()?;
        Ok(Self::new(
            active.take_offset(),
            active.take_workflow_id(),
            active_contracts,
            if active.has_trace_context() {
                Some(active.take_trace_context().into())
            } else {
                None
            },
        ))
    }
}
