use crate::data::event::DamlEvent;
use crate::data::trace::DamlTraceContext;
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::Transaction;
use crate::util;
use crate::util::Required;
use chrono::DateTime;
use chrono::Utc;
use std::convert::TryFrom;

/// A DAML ledger transaction.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlTransaction {
    pub transaction_id: String,
    pub command_id: String,
    pub workflow_id: String,
    pub effective_at: DateTime<Utc>,
    pub events: Vec<DamlEvent>,
    pub offset: String,
    pub trace_context: Option<DamlTraceContext>,
}

impl DamlTransaction {
    pub fn new(
        transaction_id: impl Into<String>,
        command_id: impl Into<String>,
        workflow_id: impl Into<String>,
        effective_at: impl Into<DateTime<Utc>>,
        events: impl Into<Vec<DamlEvent>>,
        offset: impl Into<String>,
        trace_context: impl Into<Option<DamlTraceContext>>,
    ) -> Self {
        Self {
            transaction_id: transaction_id.into(),
            command_id: command_id.into(),
            workflow_id: workflow_id.into(),
            effective_at: effective_at.into(),
            events: events.into(),
            offset: offset.into(),
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

    pub const fn effective_at(&self) -> &DateTime<Utc> {
        &self.effective_at
    }

    pub fn events(&self) -> &[DamlEvent] {
        &self.events
    }

    pub fn take_events(self) -> Vec<DamlEvent> {
        self.events
    }

    pub fn offset(&self) -> &str {
        &self.offset
    }

    pub const fn trace_context(&self) -> &Option<DamlTraceContext> {
        &self.trace_context
    }
}

impl TryFrom<Transaction> for DamlTransaction {
    type Error = DamlError;

    fn try_from(tx: Transaction) -> Result<Self, Self::Error> {
        Ok(Self::new(
            tx.transaction_id,
            tx.command_id,
            tx.workflow_id,
            util::from_grpc_timestamp(&tx.effective_at.req()?),
            tx.events.into_iter().map(DamlEvent::try_from).collect::<DamlResult<Vec<_>>>()?,
            tx.offset,
            tx.trace_context.map(DamlTraceContext::from),
        ))
    }
}
