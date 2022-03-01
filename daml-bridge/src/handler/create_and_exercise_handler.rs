use crate::aliases::{Archive, BridgeConfig, GrpcClient};
use crate::handler::common::{bad_request, extract_from_token, internal_server_error, parse_auth_header, JsonResult};
use daml_grpc::data::command::{DamlCommand, DamlCreateAndExerciseCommand};
use daml_grpc::data::DamlTransactionTree;
use daml_grpc::{CommandExecutor, DamlSimpleExecutorBuilder};
use daml_json::request::{DamlJsonCreateAndExerciseRequest, DamlJsonCreateAndExerciseResponse};
use daml_json::request_converter::JsonToGrpcRequestConverter;
use daml_json::response_converter::GrpcToJsonResponseConverter;
use daml_json::value_encode::JsonValueEncoder;

/// DOCME
pub struct CreateAndExerciseHandler {
    config: BridgeConfig,
    archive: Archive,
    client: GrpcClient,
}

impl CreateAndExerciseHandler {
    pub fn new(config: BridgeConfig, archive: Archive, client: GrpcClient) -> Self {
        Self {
            config,
            archive,
            client,
        }
    }

    /// DOCME
    pub async fn create_and_exercise(
        &self,
        create: DamlJsonCreateAndExerciseRequest,
        auth_header: Option<&str>,
    ) -> JsonResult<DamlJsonCreateAndExerciseResponse> {
        let create_command = self.make_command(&create).await?;
        let transaction = self.execute_command(create_command, auth_header).await?;
        self.make_response(&transaction)
    }

    async fn make_command(
        &self,
        create_and_exercise: &DamlJsonCreateAndExerciseRequest,
    ) -> JsonResult<DamlCreateAndExerciseCommand> {
        let archive_locked = &self.archive.read().await;
        let request_converter = JsonToGrpcRequestConverter::new(archive_locked);
        request_converter.convert_create_and_exercise_request(create_and_exercise).map_err(bad_request)
    }

    async fn execute_command(
        &self,
        command: DamlCreateAndExerciseCommand,
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
            .execute_for_transaction_tree(DamlCommand::CreateAndExercise(command))
            .await
            .map_err(internal_server_error)
    }

    fn make_response(&self, transaction: &DamlTransactionTree) -> JsonResult<DamlJsonCreateAndExerciseResponse> {
        GrpcToJsonResponseConverter::new(JsonValueEncoder::new(
            self.config.encode_decimal_as_string(),
            self.config.encode_int64_as_string(),
        ))
        .convert_create_and_exercise_response(transaction)
        .map_err(internal_server_error)
    }
}
