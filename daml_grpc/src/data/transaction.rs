use crate::data::event::DamlEvent;

use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::Transaction;
use crate::util;
use crate::util::Required;
use chrono::DateTime;
use chrono::Utc;
use std::convert::TryFrom;

/// A Daml ledger transaction.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlTransaction {
    transaction_id: String,
    command_id: String,
    workflow_id: String,
    effective_at: DateTime<Utc>,
    events: Vec<DamlEvent>,
    offset: String,
}

impl DamlTransaction {
    pub fn new(
        transaction_id: impl Into<String>,
        command_id: impl Into<String>,
        workflow_id: impl Into<String>,
        effective_at: impl Into<DateTime<Utc>>,
        events: impl Into<Vec<DamlEvent>>,
        offset: impl Into<String>,
    ) -> Self {
        Self {
            transaction_id: transaction_id.into(),
            command_id: command_id.into(),
            workflow_id: workflow_id.into(),
            effective_at: effective_at.into(),
            events: events.into(),
            offset: offset.into(),
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
        ))
    }
}
