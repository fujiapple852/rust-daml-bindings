use crate::data::event::DamlEvent;
use crate::data::trace::DamlTraceContext;
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf_autogen::event::Event;
use crate::grpc_protobuf_autogen::transaction::Transaction;
use crate::util;
use chrono::DateTime;
use chrono::Utc;
use protobuf::RepeatedField;
use std::convert::{TryFrom, TryInto};

/// A DAML ledger transaction.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlTransaction {
    transaction_id: String,
    command_id: String,
    workflow_id: String,
    effective_at: DateTime<Utc>,
    events: Vec<DamlEvent>,
    offset: String,
    trace_context: DamlTraceContext,
}

impl DamlTransaction {
    pub fn new(
        transaction_id: impl Into<String>,
        command_id: impl Into<String>,
        workflow_id: impl Into<String>,
        effective_at: impl Into<DateTime<Utc>>,
        events: impl Into<Vec<DamlEvent>>,
        offset: impl Into<String>,
        trace_context: impl Into<DamlTraceContext>,
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

    pub fn effective_at(&self) -> &DateTime<Utc> {
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

    pub fn trace_context(&self) -> &DamlTraceContext {
        &self.trace_context
    }
}

impl TryFrom<Transaction> for DamlTransaction {
    type Error = DamlError;

    fn try_from(mut tx: Transaction) -> Result<Self, Self::Error> {
        Ok(Self::new(
            tx.take_transaction_id(),
            tx.take_command_id(),
            tx.take_workflow_id(),
            util::make_datetime(&tx.take_effective_at()),
            (tx.take_events() as RepeatedField<Event>)
                .into_iter()
                .map(TryInto::try_into)
                .collect::<DamlResult<Vec<_>>>()?,
            tx.take_offset(),
            tx.take_trace_context(),
        ))
    }
}
