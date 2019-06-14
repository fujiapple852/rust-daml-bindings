use crate::data::offset::DamlLedgerOffset;
use crate::data::trace::DamlTraceContext;
use crate::data::DamlError;
use crate::grpc_protobuf_autogen::command_completion_service::Checkpoint;
use crate::grpc_protobuf_autogen::command_completion_service::CompletionStreamResponse;
use crate::grpc_protobuf_autogen::completion::Completion;
use crate::grpc_protobuf_autogen::status::Status;
use crate::util;
use chrono::DateTime;
use chrono::Utc;
use protobuf::RepeatedField;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Eq, PartialEq)]
pub struct DamlCompletionResponse {
    checkpoint: DamlCheckpoint,
    completions: Vec<DamlCompletion>,
}

impl DamlCompletionResponse {
    pub fn new(checkpoint: impl Into<DamlCheckpoint>, completions: impl Into<Vec<DamlCompletion>>) -> Self {
        Self {
            checkpoint: checkpoint.into(),
            completions: completions.into(),
        }
    }

    pub fn checkpoint(&self) -> &DamlCheckpoint {
        &self.checkpoint
    }

    pub fn completions(&self) -> &[DamlCompletion] {
        &self.completions
    }

    // TODO review this
    pub fn take_completions(self) -> Vec<DamlCompletion> {
        self.completions
    }
}

impl TryFrom<CompletionStreamResponse> for DamlCompletionResponse {
    type Error = DamlError;

    fn try_from(mut response: CompletionStreamResponse) -> Result<Self, Self::Error> {
        let checkpoint: DamlCheckpoint = response.take_checkpoint().try_into()?;
        Ok(Self::new(
            checkpoint,
            (response.take_completions() as RepeatedField<Completion>)
                .into_iter()
                .map(Into::into)
                .collect::<Vec<DamlCompletion>>(),
        ))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DamlCheckpoint {
    record_time: DateTime<Utc>,
    offset: DamlLedgerOffset,
}

impl DamlCheckpoint {
    pub fn new(record_time: impl Into<DateTime<Utc>>, offset: impl Into<DamlLedgerOffset>) -> Self {
        Self {
            record_time: record_time.into(),
            offset: offset.into(),
        }
    }

    pub fn record_time(&self) -> &DateTime<Utc> {
        &self.record_time
    }

    pub fn offset(&self) -> &DamlLedgerOffset {
        &self.offset
    }
}

impl TryFrom<Checkpoint> for DamlCheckpoint {
    type Error = DamlError;

    fn try_from(mut checkpoint: Checkpoint) -> Result<Self, Self::Error> {
        let offset: DamlLedgerOffset = checkpoint.take_offset().try_into()?;
        Ok(Self::new(util::make_datetime(&checkpoint.take_record_time()), offset))
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct DamlCompletion {
    command_id: String,
    status: DamlStatus,
    transaction_id: String,
    trace_context: Option<DamlTraceContext>,
}

impl DamlCompletion {
    pub fn new(
        command_id: impl Into<String>,
        status: impl Into<DamlStatus>,
        transaction_id: impl Into<String>,
        trace_context: Option<DamlTraceContext>,
    ) -> Self {
        Self {
            command_id: command_id.into(),
            status: status.into(),
            transaction_id: transaction_id.into(),
            trace_context,
        }
    }

    pub fn command_id(&self) -> &str {
        &self.command_id
    }

    pub fn status(&self) -> &DamlStatus {
        &self.status
    }

    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }

    pub fn trace_context(&self) -> &Option<DamlTraceContext> {
        &self.trace_context
    }
}

impl From<Completion> for DamlCompletion {
    fn from(mut completion: Completion) -> Self {
        Self::new(
            completion.take_command_id(),
            completion.take_status(),
            completion.take_transaction_id(),
            if completion.has_trace_context() {
                Some(completion.take_trace_context().into())
            } else {
                None
            },
        )
    }
}

// TODO there is a `details` field here
#[derive(Debug, Eq, PartialEq, Default)]
pub struct DamlStatus {
    code: i32,
    message: String,
}

impl DamlStatus {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> i32 {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<Status> for DamlStatus {
    fn from(status: Status) -> Self {
        Self::new(status.code, status.message)
    }
}
