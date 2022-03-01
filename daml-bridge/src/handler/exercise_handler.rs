use crate::aliases::{Archive, BridgeConfig, GrpcClient};
use crate::handler::common::{bad_request, extract_from_token, internal_server_error, parse_auth_header, JsonResult};
use daml_grpc::data::command::{DamlCommand, DamlExerciseCommand};
use daml_grpc::data::DamlTransactionTree;
use daml_grpc::{CommandExecutor, DamlSimpleExecutorBuilder};
use daml_json::request::{DamlJsonExerciseRequest, DamlJsonExerciseResponse};
use daml_json::request_converter::JsonToGrpcRequestConverter;
use daml_json::response_converter::GrpcToJsonResponseConverter;
use daml_json::value_encode::JsonValueEncoder;

/// DOCME
pub struct ExerciseHandler {
    config: BridgeConfig,
    archive: Archive,
    client: GrpcClient,
}

impl ExerciseHandler {
    pub fn new(config: BridgeConfig, archive: Archive, client: GrpcClient) -> Self {
        Self {
            config,
            archive,
            client,
        }
    }

    /// DOCME
    pub async fn exercise(
        &self,
        exercise: DamlJsonExerciseRequest,
        auth_header: Option<&str>,
    ) -> JsonResult<DamlJsonExerciseResponse> {
        let create_command = self.make_command(exercise).await?;
        let transaction = self.execute_command(create_command, auth_header).await?;
        self.make_response(&transaction)
    }

    async fn make_command(&self, exercise: DamlJsonExerciseRequest) -> JsonResult<DamlExerciseCommand> {
        let archive_locked = &self.archive.read().await;
        let request_converter = JsonToGrpcRequestConverter::new(archive_locked);
        request_converter.convert_exercise_request(&exercise).map_err(bad_request)
    }

    async fn execute_command(
        &self,
        command: DamlExerciseCommand,
        auth_header: Option<&str>,
    ) -> JsonResult<DamlTransactionTree> {
        let (token, parsed_token) = parse_auth_header(auth_header)?;
        let (acting_party, _ledger_id, application_id) = extract_from_token(&parsed_token)?;
        DamlSimpleExecutorBuilder::new(&self.client)
            .application_id(application_id)
            .act_as(acting_party)
            .auth_token(token)
            .build()
            .map_err(internal_server_error)?
            .execute_for_transaction_tree(DamlCommand::Exercise(command))
            .await
            .map_err(internal_server_error)
    }

    fn make_response(&self, transaction: &DamlTransactionTree) -> JsonResult<DamlJsonExerciseResponse> {
        GrpcToJsonResponseConverter::new(JsonValueEncoder::new(
            self.config.encode_decimal_as_string(),
            self.config.encode_int64_as_string(),
        ))
        .convert_exercise_response(transaction)
        .map_err(internal_server_error)
    }
}
