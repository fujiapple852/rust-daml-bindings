use crate::data::event::{DamlArchivedEvent, DamlCreatedEvent, DamlExercisedEvent};
use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::{event, tree_event};
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Event, TreeEvent};
use std::convert::TryFrom;

/// A Daml ledger event.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DamlEvent {
    Created(Box<DamlCreatedEvent>),
    Archived(Box<DamlArchivedEvent>),
}

impl DamlEvent {
    pub fn try_created(self) -> DamlResult<DamlCreatedEvent> {
        match self {
            DamlEvent::Created(e) => Ok(*e),
            DamlEvent::Archived(_) => Err(self.make_unexpected_type_error("Created")),
        }
    }

    pub fn try_archived(self) -> DamlResult<DamlArchivedEvent> {
        match self {
            DamlEvent::Archived(e) => Ok(*e),
            DamlEvent::Created(_) => Err(self.make_unexpected_type_error("Archived")),
        }
    }

    /// The name of this [`DamlEvent`] variant type.
    pub fn variant_name(&self) -> &str {
        match self {
            DamlEvent::Created(_) => "Created",
            DamlEvent::Archived(_) => "Archived",
        }
    }

    pub fn event_id(&self) -> &str {
        match self {
            DamlEvent::Created(c) => c.event_id(),
            DamlEvent::Archived(a) => a.event_id(),
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
                        event::Event::Created(e) => DamlEvent::Created(Box::new(DamlCreatedEvent::try_from(e)?)),
                        event::Event::Archived(e) => DamlEvent::Archived(Box::new(DamlArchivedEvent::try_from(e)?)),
                    })
                };
                convert(e)
            },
            None => Err(DamlError::new_failed_conversion("GRPC Event was None")),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DamlTreeEvent {
    Created(DamlCreatedEvent),
    Exercised(DamlExercisedEvent),
}

impl DamlTreeEvent {
    pub fn event_id(&self) -> &str {
        match self {
            DamlTreeEvent::Created(c) => c.event_id(),
            DamlTreeEvent::Exercised(e) => e.event_id(),
        }
    }
}

impl TryFrom<TreeEvent> for DamlTreeEvent {
    type Error = DamlError;

    fn try_from(event: TreeEvent) -> Result<Self, Self::Error> {
        match event.kind {
            Some(e) => {
                let convert = |sum| {
                    Ok(match sum {
                        tree_event::Kind::Created(e) => DamlTreeEvent::Created(DamlCreatedEvent::try_from(e)?),
                        tree_event::Kind::Exercised(e) => DamlTreeEvent::Exercised(DamlExercisedEvent::try_from(e)?),
                    })
                };
                convert(e)
            },
            None => Err(DamlError::new_failed_conversion("GRPC TreeEvent was None")),
        }
    }
}
