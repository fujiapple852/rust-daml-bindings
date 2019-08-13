use crate::data::event::archived::DamlArchivedEvent;
use crate::data::event::created::DamlCreatedEvent;
use crate::data::event::exercised::DamlExercisedEvent;
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf_autogen::event::Event;
use crate::grpc_protobuf_autogen::event::Event_oneof_event;
use crate::grpc_protobuf_autogen::transaction::TreeEvent;
use crate::grpc_protobuf_autogen::transaction::TreeEvent_oneof_kind;
use std::convert::{TryFrom, TryInto};

/// A DAML ledger event.
#[derive(Debug, Eq, PartialEq)]
pub enum DamlEvent {
    Created(Box<DamlCreatedEvent>),
    Archived(Box<DamlArchivedEvent>),
}

impl DamlEvent {
    pub fn try_created(self) -> DamlResult<DamlCreatedEvent> {
        match self {
            DamlEvent::Created(e) => Ok(*e),
            _ => Err(self.make_unexpected_type_error("Created")),
        }
    }

    pub fn try_archived(self) -> DamlResult<DamlArchivedEvent> {
        match self {
            DamlEvent::Archived(e) => Ok(*e),
            _ => Err(self.make_unexpected_type_error("Archived")),
        }
    }

    /// The name of this [`DamlEvent`] variant type.
    pub fn variant_name(&self) -> &str {
        match self {
            DamlEvent::Created(_) => "Created",
            DamlEvent::Archived(_) => "Archived",
        }
    }

    fn make_unexpected_type_error(&self, expected: &str) -> DamlError {
        DamlError::UnexpectedType(expected.to_owned(), self.variant_name().to_owned())
    }
}

impl TryFrom<Event> for DamlEvent {
    type Error = DamlError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event.event {
            Some(e) => {
                let convert = |sum| {
                    Ok(match sum {
                        Event_oneof_event::created(e) => DamlEvent::Created(Box::new(e.try_into()?)),
                        Event_oneof_event::archived(e) => DamlEvent::Archived(Box::new(e.into())),
                    })
                };
                convert(e)
            },
            None => Err(DamlError::new_failed_conversion("GRPC Event was None")),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum DamlTreeEvent {
    Created(DamlCreatedEvent),
    Exercised(DamlExercisedEvent),
}

impl TryFrom<TreeEvent> for DamlTreeEvent {
    type Error = DamlError;

    fn try_from(event: TreeEvent) -> Result<Self, Self::Error> {
        match event.kind {
            Some(e) => {
                let convert = |sum| {
                    Ok(match sum {
                        TreeEvent_oneof_kind::created(e) => DamlTreeEvent::Created(e.try_into()?),
                        TreeEvent_oneof_kind::exercised(e) => DamlTreeEvent::Exercised(e.try_into()?),
                    })
                };
                convert(e)
            },
            None => Err(DamlError::new_failed_conversion("GRPC TreeEvent was None")),
        }
    }
}
