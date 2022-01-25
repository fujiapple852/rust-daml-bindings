use crate::data::value::DamlValue;
use crate::data::{DamlError, DamlIdentifier};
use crate::grpc_protobuf::com::daml::ledger::api::v1::ExercisedEvent;
use crate::util::Required;
use std::convert::TryFrom;

/// An event which represents exercising of a choice on a contract on a Daml ledger.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlExercisedEvent {
    event_id: String,
    contract_id: String,
    template_id: DamlIdentifier,
    choice: String,
    choice_argument: DamlValue,
    acting_parties: Vec<String>,
    consuming: bool,
    witness_parties: Vec<String>,
    child_event_ids: Vec<String>,
    exercise_result: DamlValue,
}

impl DamlExercisedEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_id: impl Into<String>,
        contract_id: impl Into<String>,
        template_id: impl Into<DamlIdentifier>,
        choice: impl Into<String>,
        choice_argument: impl Into<DamlValue>,
        acting_parties: impl Into<Vec<String>>,
        consuming: bool,
        witness_parties: impl Into<Vec<String>>,
        child_event_ids: impl Into<Vec<String>>,
        exercise_result: impl Into<DamlValue>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            contract_id: contract_id.into(),
            template_id: template_id.into(),
            choice: choice.into(),
            choice_argument: choice_argument.into(),
            acting_parties: acting_parties.into(),
            consuming,
            witness_parties: witness_parties.into(),
            child_event_ids: child_event_ids.into(),
            exercise_result: exercise_result.into(),
        }
    }

    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn contract_id(&self) -> &str {
        &self.contract_id
    }

    pub const fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    pub fn choice(&self) -> &str {
        &self.choice
    }

    pub const fn choice_argument(&self) -> &DamlValue {
        &self.choice_argument
    }

    pub fn acting_parties(&self) -> &[String] {
        &self.acting_parties
    }

    pub const fn consuming(&self) -> bool {
        self.consuming
    }

    pub fn witness_parties(&self) -> &[String] {
        &self.witness_parties
    }

    pub fn child_event_ids(&self) -> &[String] {
        &self.child_event_ids
    }

    pub const fn exercise_result(&self) -> &DamlValue {
        &self.exercise_result
    }

    pub fn take_exercise_result(self) -> DamlValue {
        self.exercise_result
    }
}

impl TryFrom<ExercisedEvent> for DamlExercisedEvent {
    type Error = DamlError;

    fn try_from(event: ExercisedEvent) -> Result<Self, Self::Error> {
        Ok(Self::new(
            event.event_id,
            event.contract_id,
            event.template_id.req().map(DamlIdentifier::from)?,
            event.choice,
            event.choice_argument.req().and_then(DamlValue::try_from)?,
            event.acting_parties,
            event.consuming,
            event.witness_parties,
            event.child_event_ids,
            event.exercise_result.req().and_then(DamlValue::try_from)?,
        ))
    }
}
