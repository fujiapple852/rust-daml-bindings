use crate::data::event::DamlTreeEvent;
use crate::data::trace::DamlTraceContext;
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::TransactionTree;
use crate::util;
use crate::util::Required;
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;
use std::convert::TryFrom;

/// A DAML ledger transaction tree.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlTransactionTree {
    pub transaction_id: String,
    pub command_id: String,
    pub workflow_id: String,
    pub effective_at: DateTime<Utc>,
    pub offset: String,
    pub events_by_id: HashMap<String, DamlTreeEvent>,
    pub root_event_ids: Vec<String>,
    pub trace_context: Option<DamlTraceContext>,
}

impl DamlTransactionTree {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transaction_id: impl Into<String>,
        command_id: impl Into<String>,
        workflow_id: impl Into<String>,
        effective_at: impl Into<DateTime<Utc>>,
        offset: impl Into<String>,
        events_by_id: impl Into<HashMap<String, DamlTreeEvent>>,
        root_event_ids: impl Into<Vec<String>>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> Self {
        Self {
            transaction_id: transaction_id.into(),
            command_id: command_id.into(),
            workflow_id: workflow_id.into(),
            effective_at: effective_at.into(),
            offset: offset.into(),
            events_by_id: events_by_id.into(),
            root_event_ids: root_event_ids.into(),
            trace_context: trace_context.into(),
        }
    }

    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }

    pub fn command_id(&self) -> &str {
        &self.command_id
    }

    pub fn workflow_id(&self) -> &str {
        &self.workflow_id
    }

    pub fn effective_at(&self) -> &DateTime<Utc> {
        &self.effective_at
    }

    pub fn events_by_id(&self) -> &HashMap<String, DamlTreeEvent> {
        &self.events_by_id
    }

    pub fn take_events_by_id(self) -> HashMap<String, DamlTreeEvent> {
        self.events_by_id
    }

    pub fn root_event_ids(&self) -> &[String] {
        &self.root_event_ids
    }

    pub fn offset(&self) -> &str {
        &self.offset
    }

    pub fn trace_context(&self) -> &Option<DamlTraceContext> {
        &self.trace_context
    }
}

impl TryFrom<TransactionTree> for DamlTransactionTree {
    type Error = DamlError;

    fn try_from(tx: TransactionTree) -> Result<Self, Self::Error> {
        let events_by_id = tx
            .events_by_id
            .into_iter()
            .map(|(id, event)| match DamlTreeEvent::try_from(event) {
                Ok(m) => Ok((id, m)),
                Err(e) => Err(e),
            })
            .collect::<DamlResult<HashMap<_, _>>>()?;
        Ok(Self::new(
            tx.transaction_id,
            tx.command_id,
            tx.workflow_id,
            util::from_grpc_timestamp(&tx.effective_at.req()?),
            tx.offset,
            events_by_id,
            tx.root_event_ids,
            tx.trace_context.map(DamlTraceContext::from),
        ))
    }
}
