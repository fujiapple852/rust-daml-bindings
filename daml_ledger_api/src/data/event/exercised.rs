use crate::data::identifier::DamlIdentifier;
use crate::data::value::DamlValue;
use crate::data::DamlError;
use crate::grpc_protobuf_autogen::event::ExercisedEvent;
use std::convert::{TryFrom, TryInto};

/// An event which represents exercising of a choice on a contract on a DAML ledger.
#[derive(Debug, Eq, PartialEq)]
pub struct DamlExercisedEvent {
    event_id: String,
    contract_id: String,
    template_id: DamlIdentifier,
    contract_creating_event_id: String,
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
        contract_creating_event_id: impl Into<String>,
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
            contract_creating_event_id: contract_creating_event_id.into(),
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

    pub fn template_id(&self) -> &DamlIdentifier {
        &self.template_id
    }

    pub fn contract_creating_event_id(&self) -> &str {
        &self.contract_creating_event_id
    }

    pub fn choice(&self) -> &str {
        &self.choice
    }

    pub fn choice_argument(&self) -> &DamlValue {
        &self.choice_argument
    }

    pub fn acting_parties(&self) -> &[String] {
        &self.acting_parties
    }

    pub fn consuming(&self) -> bool {
        self.consuming
    }

    pub fn witness_parties(&self) -> &[String] {
        &self.witness_parties
    }

    pub fn child_event_ids(&self) -> &[String] {
        &self.child_event_ids
    }

    pub fn exercise_result(&self) -> &DamlValue {
        &self.exercise_result
    }
}

impl TryFrom<ExercisedEvent> for DamlExercisedEvent {
    type Error = DamlError;

    fn try_from(mut event: ExercisedEvent) -> Result<Self, Self::Error> {
        let value: DamlValue = event.take_choice_argument().try_into()?;
        let exercise_result: DamlValue = event.take_exercise_result().try_into()?;
        Ok(Self::new(
            event.take_event_id(),
            event.take_contract_id(),
            event.take_template_id(),
            event.take_contract_creating_event_id(),
            event.take_choice(),
            value,
            event.take_acting_parties(),
            event.get_consuming(),
            event.take_witness_parties(),
            event.take_child_event_ids(),
            exercise_result,
        ))
    }
}
