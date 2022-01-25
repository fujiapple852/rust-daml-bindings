use crate::data::offset::DamlLedgerOffset;

use crate::data::{DamlError, DamlResult};
use crate::grpc_protobuf::com::daml::ledger::api::v1::completion::DeduplicationPeriod;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Checkpoint, Completion, CompletionStreamResponse};
use crate::grpc_protobuf::google::rpc::Status;
use crate::util;
use crate::util::Required;
use chrono::DateTime;
use chrono::Utc;
use std::convert::TryFrom;
use std::time::Duration;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DamlCompletionResponse {
    checkpoint: Option<DamlCheckpoint>,
    completions: Vec<DamlCompletion>,
}

impl DamlCompletionResponse {
    pub fn new(checkpoint: impl Into<Option<DamlCheckpoint>>, completions: impl Into<Vec<DamlCompletion>>) -> Self {
        Self {
            checkpoint: checkpoint.into(),
            completions: completions.into(),
        }
    }

    pub const fn checkpoint(&self) -> &Option<DamlCheckpoint> {
        &self.checkpoint
    }

    pub fn completions(&self) -> &[DamlCompletion] {
        &self.completions
    }

    pub fn take_completions(self) -> Vec<DamlCompletion> {
        self.completions
    }
}

impl TryFrom<CompletionStreamResponse> for DamlCompletionResponse {
    type Error = DamlError;

    fn try_from(response: CompletionStreamResponse) -> Result<Self, Self::Error> {
        let checkpoint = response.checkpoint.map(DamlCheckpoint::try_from).transpose()?;
        let completions =
            response.completions.into_iter().map(DamlCompletion::try_from).collect::<DamlResult<Vec<_>>>()?;
        Ok(Self::new(checkpoint, completions))
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
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

    pub const fn record_time(&self) -> &DateTime<Utc> {
        &self.record_time
    }

    pub const fn offset(&self) -> &DamlLedgerOffset {
        &self.offset
    }
}

impl TryFrom<Checkpoint> for DamlCheckpoint {
    type Error = DamlError;

    fn try_from(checkpoint: Checkpoint) -> Result<Self, Self::Error> {
        let record_time = util::from_grpc_timestamp(&checkpoint.record_time.req()?);
        let offset = DamlLedgerOffset::try_from(checkpoint.offset.req()?)?;
        Ok(Self::new(record_time, offset))
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct DamlCompletion {
    command_id: String,
    status: DamlStatus,
    transaction_id: String,
    application_id: String,
    act_as: Vec<String>,
    submission_id: String,
    deduplication_period: Option<DamlCompletionDeduplicationPeriod>,
}

impl DamlCompletion {
    pub fn new(
        command_id: impl Into<String>,
        status: impl Into<DamlStatus>,
        transaction_id: impl Into<String>,
        application_id: impl Into<String>,
        act_as: impl Into<Vec<String>>,
        submission_id: impl Into<String>,
        deduplication_period: impl Into<Option<DamlCompletionDeduplicationPeriod>>,
    ) -> Self {
        Self {
            command_id: command_id.into(),
            status: status.into(),
            transaction_id: transaction_id.into(),
            application_id: application_id.into(),
            act_as: act_as.into(),
            submission_id: submission_id.into(),
            deduplication_period: deduplication_period.into(),
        }
    }

    pub fn command_id(&self) -> &str {
        &self.command_id
    }

    pub const fn status(&self) -> &DamlStatus {
        &self.status
    }

    pub fn transaction_id(&self) -> &str {
        &self.transaction_id
    }
}

impl TryFrom<Completion> for DamlCompletion {
    type Error = DamlError;

    fn try_from(completion: Completion) -> DamlResult<Self> {
        Ok(Self::new(
            completion.command_id,
            // The protobuf field `Completion.status` is documented as being optional but is treated as mandatory here
            // as it is unclear what the absence of this field implies.  An alternative solution would be to create a
            // special "unknown" `DamlStatus` value, perhaps by reworking `DamlStatus` as an enum.
            DamlStatus::from(completion.status.req()?),
            completion.transaction_id,
            completion.application_id,
            completion.act_as,
            completion.submission_id,
            completion.deduplication_period.map(DamlCompletionDeduplicationPeriod::from),
        ))
    }
}

// TODO there is a `details` field here
#[derive(Debug, Eq, PartialEq, Clone, Default)]
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

    pub const fn code(&self) -> i32 {
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

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DamlCompletionDeduplicationPeriod {
    DeduplicationOffset(String),
    DeduplicationDuration(Duration),
}

impl From<DeduplicationPeriod> for DamlCompletionDeduplicationPeriod {
    fn from(deduplication_period: DeduplicationPeriod) -> Self {
        match deduplication_period {
            DeduplicationPeriod::DeduplicationOffset(offset) =>
                DamlCompletionDeduplicationPeriod::DeduplicationOffset(offset),
            DeduplicationPeriod::DeduplicationDuration(duration) =>
                DamlCompletionDeduplicationPeriod::DeduplicationDuration(util::from_grpc_duration(&duration)),
        }
    }
}
