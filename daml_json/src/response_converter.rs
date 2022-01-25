use crate::data::{DamlJsonArchivedEvent, DamlJsonCreatedEvent, DamlJsonEvent, DamlJsonExerciseResult};
use crate::error::{DamlJsonReqConError, DamlJsonReqConResult};
use crate::request::{
    DamlJsonCreateAndExerciseResponse, DamlJsonCreateResponse, DamlJsonExerciseByKeyResponse, DamlJsonExerciseResponse,
};
use crate::value_encode::JsonValueEncoder;
use daml_grpc::data::event::{DamlCreatedEvent, DamlEvent, DamlExercisedEvent, DamlTreeEvent};
use daml_grpc::data::{DamlTransaction, DamlTransactionTree};

/// Convert a GRPC API transaction response to a JSON API response.
#[derive(Debug)]
pub struct GrpcToJsonResponseConverter {
    encoder: JsonValueEncoder,
}

impl GrpcToJsonResponseConverter {
    pub const fn new(encoder: JsonValueEncoder) -> Self {
        Self {
            encoder,
        }
    }

    /// Convert a [`DamlTransaction`] to a [`DamlJsonCreateResponse`].
    pub fn convert_create_response(
        &self,
        transaction: &DamlTransaction,
    ) -> DamlJsonReqConResult<DamlJsonCreateResponse> {
        if transaction.events().len() != 1 {
            Err(DamlJsonReqConError::UnexpectedGrpcEvent)
        } else if let Some(DamlEvent::Created(created)) = transaction.events().first() {
            Ok(DamlJsonCreateResponse {
                status: 200,
                result: self.created_event(created)?,
                warnings: None,
            })
        } else {
            Err(DamlJsonReqConError::UnexpectedGrpcEvent)
        }
    }

    /// Convert a [`DamlTransactionTree`] to a [`DamlJsonExerciseResponse`].
    pub fn convert_exercise_response(
        &self,
        transaction: &DamlTransactionTree,
    ) -> DamlJsonReqConResult<DamlJsonExerciseResponse> {
        Ok(DamlJsonExerciseResponse {
            status: 200,
            result: self.exercise_result(transaction)?,
            warnings: None,
        })
    }

    /// Convert a [`DamlTransactionTree`] to a [`DamlJsonExerciseByKeyResponse`].
    pub fn convert_exercise_by_key_response(
        &self,
        transaction: &DamlTransactionTree,
    ) -> DamlJsonReqConResult<DamlJsonExerciseByKeyResponse> {
        Ok(DamlJsonExerciseByKeyResponse {
            status: 200,
            result: self.exercise_result(transaction)?,
            warnings: None,
        })
    }

    /// Convert a [`DamlTransactionTree`] to a [`DamlJsonCreateAndExerciseResponse`].
    pub fn convert_create_and_exercise_response(
        &self,
        transaction: &DamlTransactionTree,
    ) -> DamlJsonReqConResult<DamlJsonCreateAndExerciseResponse> {
        Ok(DamlJsonCreateAndExerciseResponse {
            status: 200,
            result: self.exercise_result(transaction)?,
            warnings: None,
        })
    }

    /// Convert a [`DamlTransactionTree`] to a [`DamlJsonExerciseResult`].
    ///
    /// Note that the [`DamlTransactionTree`] contains only Created and Exercised events, it does not contain Archived
    /// events however the [`DamlJsonExerciseResult`] specification says it contains 'zero or many
    /// `{"archived": {...}}` elements'.
    ///
    /// This implementation will include a single Archived event, for the contract on which the choice was executed, if
    /// the choice is consuming.  It will not return any other Archived events that may have occurred as part of the
    /// execution of the choice.
    ///
    /// This behaviour is consistent with the reference DAML SDK bridge behaviour.
    fn exercise_result(&self, transaction: &DamlTransactionTree) -> DamlJsonReqConResult<DamlJsonExerciseResult> {
        let exercise_event = first_exercised_event(transaction)?;
        let exercise_result = self.encoder.encode_value(exercise_event.exercise_result())?;
        let archive_event = derive_archived_event(exercise_event);
        let events = self.extract_created_events(transaction, archive_event)?;
        Ok(DamlJsonExerciseResult {
            exercise_result,
            events,
        })
    }

    fn extract_created_events(
        &self,
        transaction: &DamlTransactionTree,
        archive_event: Option<DamlJsonEvent>,
    ) -> DamlJsonReqConResult<Vec<DamlJsonEvent>> {
        transaction
            .events_by_id()
            .iter()
            .filter_map(|(_, e)| {
                if let DamlTreeEvent::Created(created) = e {
                    Some(self.created_event(created))
                } else {
                    None
                }
            })
            .map(|evt| evt.map(DamlJsonEvent::Created))
            .chain(archive_event.into_iter().map(Ok))
            .collect::<DamlJsonReqConResult<Vec<_>>>()
    }

    fn created_event(&self, created: &DamlCreatedEvent) -> DamlJsonReqConResult<DamlJsonCreatedEvent> {
        let observers = created.observers().to_vec();
        let agreement_text = created.agreement_text().to_owned();
        let signatories = created.signatories().to_vec();
        let contract_id = created.contract_id().to_owned();
        let template_id = created.template_id().to_string();
        let payload = self.encoder.encode_record(created.create_arguments())?;
        Ok(DamlJsonCreatedEvent::new(observers, agreement_text, payload, signatories, contract_id, template_id))
    }
}

/// Derive a [`DamlJsonEvent::Archived`] from a consuming [`DamlExercisedEvent`], return None otherwise.
fn derive_archived_event(exercise_event: &DamlExercisedEvent) -> Option<DamlJsonEvent> {
    exercise_event.consuming().then(|| {
        DamlJsonEvent::Archived(DamlJsonArchivedEvent::new(
            exercise_event.contract_id().to_owned(),
            exercise_event.template_id().to_string(),
        ))
    })
}

fn first_exercised_event(transaction: &DamlTransactionTree) -> DamlJsonReqConResult<&DamlExercisedEvent> {
    transaction
        .root_event_ids()
        .iter()
        .find_map(|r| match transaction.events_by_id().get(r) {
            Some(DamlTreeEvent::Exercised(exercised)) => Some(exercised),
            _ => None,
        })
        .ok_or(DamlJsonReqConError::MissingExercisedEvent)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::util::Required;
    use anyhow::Result;
    use chrono::{DateTime, Utc};
    use daml_grpc::data::value::{DamlRecord, DamlRecordBuilder, DamlValue};
    use daml_grpc::data::DamlIdentifier;
    use maplit::hashmap;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_convert_create_response() -> Result<()> {
        let value_encoder = JsonValueEncoder::new(false, false);
        let converter = GrpcToJsonResponseConverter::new(value_encoder);
        let dummy_tx = make_dummy_transaction()?;
        let converted = converter.convert_create_response(&dummy_tx)?;
        let serialized = serde_json::to_value(converted)?;
        assert_eq!(json!(200), *serialized.pointer("/status").req()?);
        assert_eq!(json!("ObsParty"), *serialized.pointer("/result/observers/0").req()?);
        assert_eq!(json!("agreement"), *serialized.pointer("/result/agreementText").req()?);
        assert_eq!(json!("SigParty1"), *serialized.pointer("/result/signatories/0").req()?);
        assert_eq!(json!("SigParty2"), *serialized.pointer("/result/signatories/1").req()?);
        assert_eq!(json!(false), *serialized.pointer("/result/payload/foo").req()?);
        assert_eq!(json!("something"), *serialized.pointer("/result/payload/bar").req()?);
        assert_eq!(json!("contract_id_create"), *serialized.pointer("/result/contractId").req()?);
        assert_eq!(json!("package:module:entity"), *serialized.pointer("/result/templateId").req()?);
        Ok(())
    }

    #[test]
    fn test_convert_exercise_response() -> Result<()> {
        let value_encoder = JsonValueEncoder::new(false, false);
        let converter = GrpcToJsonResponseConverter::new(value_encoder);
        let dummy_tx_tree = make_dummy_transaction_tree()?;
        let converted = converter.convert_exercise_response(&dummy_tx_tree)?;
        let ser = serde_json::to_value(converted)?;
        assert_eq!(json!(200), *ser.pointer("/status").req()?);
        assert_eq!(json!("ObsParty"), *ser.pointer("/result/events/0/created/observers/0").req()?);
        assert_eq!(json!("agreement"), *ser.pointer("/result/events/0/created/agreementText").req()?);
        assert_eq!(json!("SigParty1"), *ser.pointer("/result/events/0/created/signatories/0").req()?);
        assert_eq!(json!("SigParty2"), *ser.pointer("/result/events/0/created/signatories/1").req()?);
        assert_eq!(json!(false), *ser.pointer("/result/events/0/created/payload/foo").req()?);
        assert_eq!(json!("something"), *ser.pointer("/result/events/0/created/payload/bar").req()?);
        assert_eq!(json!("contract_id_create"), *ser.pointer("/result/events/0/created/contractId").req()?);
        assert_eq!(json!("package:module:entity"), *ser.pointer("/result/events/0/created/templateId").req()?);
        assert_eq!(json!("contract_id_exercise"), *ser.pointer("/result/events/1/archived/contractId").req()?);
        assert_eq!(json!("package:module:entity"), *ser.pointer("/result/events/1/archived/templateId").req()?);
        assert_eq!(json!(false), *ser.pointer("/result/exerciseResult/foo").req()?);
        assert_eq!(json!("something"), *ser.pointer("/result/exerciseResult/bar").req()?);
        Ok(())
    }

    #[test]
    fn test_convert_exercise_by_key_response() -> Result<()> {
        let value_encoder = JsonValueEncoder::new(false, false);
        let converter = GrpcToJsonResponseConverter::new(value_encoder);
        let dummy_tx_tree = make_dummy_transaction_tree()?;
        let converted = converter.convert_exercise_by_key_response(&dummy_tx_tree)?;
        let ser = serde_json::to_value(converted)?;
        assert_eq!(json!(200), *ser.pointer("/status").req()?);
        assert_eq!(json!("ObsParty"), *ser.pointer("/result/events/0/created/observers/0").req()?);
        assert_eq!(json!("agreement"), *ser.pointer("/result/events/0/created/agreementText").req()?);
        assert_eq!(json!("SigParty1"), *ser.pointer("/result/events/0/created/signatories/0").req()?);
        assert_eq!(json!("SigParty2"), *ser.pointer("/result/events/0/created/signatories/1").req()?);
        assert_eq!(json!(false), *ser.pointer("/result/events/0/created/payload/foo").req()?);
        assert_eq!(json!("something"), *ser.pointer("/result/events/0/created/payload/bar").req()?);
        assert_eq!(json!("contract_id_create"), *ser.pointer("/result/events/0/created/contractId").req()?);
        assert_eq!(json!("package:module:entity"), *ser.pointer("/result/events/0/created/templateId").req()?);
        assert_eq!(json!("contract_id_exercise"), *ser.pointer("/result/events/1/archived/contractId").req()?);
        assert_eq!(json!("package:module:entity"), *ser.pointer("/result/events/1/archived/templateId").req()?);
        assert_eq!(json!(false), *ser.pointer("/result/exerciseResult/foo").req()?);
        assert_eq!(json!("something"), *ser.pointer("/result/exerciseResult/bar").req()?);
        Ok(())
    }

    // TODO should create two contracts, the initial create and a create as a result of the choice
    #[test]
    fn test_convert_create_and_exercise_response() -> Result<()> {
        let value_encoder = JsonValueEncoder::new(false, false);
        let converter = GrpcToJsonResponseConverter::new(value_encoder);
        let dummy_tx_tree = make_dummy_transaction_tree()?;
        let converted = converter.convert_create_and_exercise_response(&dummy_tx_tree)?;
        let ser = serde_json::to_value(converted)?;
        assert_eq!(json!(200), *ser.pointer("/status").req()?);
        assert_eq!(json!("ObsParty"), *ser.pointer("/result/events/0/created/observers/0").req()?);
        assert_eq!(json!("agreement"), *ser.pointer("/result/events/0/created/agreementText").req()?);
        assert_eq!(json!("SigParty1"), *ser.pointer("/result/events/0/created/signatories/0").req()?);
        assert_eq!(json!("SigParty2"), *ser.pointer("/result/events/0/created/signatories/1").req()?);
        assert_eq!(json!(false), *ser.pointer("/result/events/0/created/payload/foo").req()?);
        assert_eq!(json!("something"), *ser.pointer("/result/events/0/created/payload/bar").req()?);
        assert_eq!(json!("contract_id_create"), *ser.pointer("/result/events/0/created/contractId").req()?);
        assert_eq!(json!("package:module:entity"), *ser.pointer("/result/events/0/created/templateId").req()?);
        assert_eq!(json!("contract_id_exercise"), *ser.pointer("/result/events/1/archived/contractId").req()?);
        assert_eq!(json!("package:module:entity"), *ser.pointer("/result/events/1/archived/templateId").req()?);
        assert_eq!(json!(false), *ser.pointer("/result/exerciseResult/foo").req()?);
        assert_eq!(json!("something"), *ser.pointer("/result/exerciseResult/bar").req()?);
        Ok(())
    }

    #[test]
    fn test_test_convert_exercise_response_missing_event_err() -> Result<()> {
        let value_encoder = JsonValueEncoder::new(false, false);
        let converter = GrpcToJsonResponseConverter::new(value_encoder);
        let dummy_tx_tree = make_dummy_transaction_tree_no_exercised_event()?;
        match converter.convert_exercise_response(&dummy_tx_tree) {
            Err(DamlJsonReqConError::MissingExercisedEvent) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    #[test]
    fn test_test_convert_create_response_zero_create_events_err() -> Result<()> {
        let value_encoder = JsonValueEncoder::new(false, false);
        let converter = GrpcToJsonResponseConverter::new(value_encoder);
        let dummy_tx = make_dummy_transaction_empty()?;
        match converter.convert_create_response(&dummy_tx) {
            Err(DamlJsonReqConError::UnexpectedGrpcEvent) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    #[test]
    fn test_test_convert_create_response_two_create_events_err() -> Result<()> {
        let value_encoder = JsonValueEncoder::new(false, false);
        let converter = GrpcToJsonResponseConverter::new(value_encoder);
        let dummy_tx = make_dummy_transaction_two()?;
        match converter.convert_create_response(&dummy_tx) {
            Err(DamlJsonReqConError::UnexpectedGrpcEvent) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    fn make_dummy_grpc_record() -> DamlRecord {
        DamlRecordBuilder::new()
            .add_field("foo", DamlValue::new_bool(false))
            .add_field("bar", DamlValue::new_text("something"))
            .build()
    }

    fn make_dummy_grpc_created_event_inner(record: DamlRecord) -> DamlCreatedEvent {
        DamlCreatedEvent::new(
            "event_id",
            "contract_id_create",
            DamlIdentifier::new("package", "module", "entity"),
            None,
            record,
            vec!["WitnessParty".into()],
            vec!["SigParty1".into(), "SigParty2".into()],
            vec!["ObsParty".into()],
            "agreement",
        )
    }

    fn make_dummy_grpc_created_event(record: DamlRecord) -> DamlEvent {
        DamlEvent::Created(Box::new(make_dummy_grpc_created_event_inner(record)))
    }

    fn make_dummy_grpc_created_tree_event(record: DamlRecord) -> DamlTreeEvent {
        DamlTreeEvent::Created(make_dummy_grpc_created_event_inner(record))
    }

    fn make_dummy_grpc_exercised_tree_event(args: DamlValue, result: DamlValue) -> DamlTreeEvent {
        DamlTreeEvent::Exercised(DamlExercisedEvent::new(
            "exercised_event_id",
            "contract_id_exercise",
            DamlIdentifier::new("package", "module", "entity"),
            "choice_name",
            args,
            vec!["ActingParty".into()],
            true,
            vec!["ObsParty".into()],
            vec![],
            result,
        ))
    }

    fn make_dummy_grpc_transaction(events: Vec<DamlEvent>) -> Result<DamlTransaction> {
        Ok(DamlTransaction::new(
            "tx_id",
            "cmd_id",
            "wf_id",
            "2019-01-02T03:45:56Z".parse::<DateTime<Utc>>()?,
            events,
            "offset",
        ))
    }

    fn make_dummy_grpc_transaction_tree(events: HashMap<String, DamlTreeEvent>) -> Result<DamlTransactionTree> {
        let roots = events.keys().cloned().collect::<Vec<_>>();
        Ok(DamlTransactionTree::new(
            "tx_id",
            "cmd_id",
            "wf_id",
            "2019-01-02T03:45:56Z".parse::<DateTime<Utc>>()?,
            "offset",
            events,
            roots,
        ))
    }

    fn make_dummy_transaction() -> Result<DamlTransaction> {
        let event = make_dummy_grpc_created_event(make_dummy_grpc_record());
        make_dummy_grpc_transaction(vec![event])
    }

    fn make_dummy_transaction_tree() -> Result<DamlTransactionTree> {
        let record = make_dummy_grpc_record();
        let created_event = make_dummy_grpc_created_tree_event(record);
        let choice_result = make_dummy_grpc_record();
        let exercised_event =
            make_dummy_grpc_exercised_tree_event(DamlValue::Unit, DamlValue::new_record(choice_result));
        make_dummy_grpc_transaction_tree(hashmap!(
        exercised_event.event_id().into() => exercised_event,
        created_event.event_id().into() => created_event))
    }

    fn make_dummy_transaction_tree_no_exercised_event() -> Result<DamlTransactionTree> {
        let record = make_dummy_grpc_record();
        let created_event = make_dummy_grpc_created_tree_event(record);
        make_dummy_grpc_transaction_tree(hashmap!(created_event.event_id().into() => created_event))
    }

    fn make_dummy_transaction_empty() -> Result<DamlTransaction> {
        make_dummy_grpc_transaction(vec![])
    }

    fn make_dummy_transaction_two() -> Result<DamlTransaction> {
        let event1 = make_dummy_grpc_created_event(make_dummy_grpc_record());
        let event2 = make_dummy_grpc_created_event(make_dummy_grpc_record());
        make_dummy_grpc_transaction(vec![event1, event2])
    }
}
