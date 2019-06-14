use grpcio::Channel;

use crate::data::DamlError;
use crate::data::DamlResult;

use crate::grpc_protobuf_autogen::empty::Empty;
use crate::grpc_protobuf_autogen::time_service::GetTimeRequest;
use crate::grpc_protobuf_autogen::time_service::GetTimeResponse;
use crate::grpc_protobuf_autogen::time_service::SetTimeRequest;
use crate::grpc_protobuf_autogen::time_service_grpc::TimeServiceClient;
use crate::util;
use chrono::DateTime;
use chrono::Utc;
use futures::Future;
use futures::Stream;
use grpcio::ClientSStreamReceiver;
use grpcio::ClientUnaryReceiver;

/// Get and set the time of a DAML ledger (requires `testing` feature).
pub struct DamlTimeService {
    grpc_client: TimeServiceClient,
    ledger_id: String,
}

impl DamlTimeService {
    pub fn new(channel: Channel, ledger_id: String) -> Self {
        Self {
            grpc_client: TimeServiceClient::new(channel),
            ledger_id,
        }
    }

    pub fn get_time(&self) -> DamlResult<impl Stream<Item = DateTime<Utc>, Error = DamlError>> {
        let mut request = GetTimeRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        let async_response: ClientSStreamReceiver<GetTimeResponse> = self.grpc_client.get_time(&request)?;
        Ok(async_response.map_err(Into::into).map(|r: GetTimeResponse| util::make_datetime(r.get_current_time())))
    }

    pub fn set_time(
        &self,
        current_time: impl Into<DateTime<Utc>>,
        new_time: impl Into<DateTime<Utc>>,
    ) -> DamlResult<impl Future<Item = (), Error = DamlError>> {
        let mut request = SetTimeRequest::new();
        request.set_ledger_id(self.ledger_id.clone());
        request.set_current_time(util::make_timestamp_secs(current_time.into()));
        request.set_new_time(util::make_timestamp_secs(new_time.into()));
        let async_result: ClientUnaryReceiver<Empty> = self.grpc_client.set_time_async(&request)?;
        Ok(async_result.map_err(Into::into).map(|_| ()))
    }

    pub fn set_time_sync(&self, current_time: DateTime<Utc>, new_time: DateTime<Utc>) -> DamlResult<()> {
        self.set_time(current_time, new_time)?.wait()
    }
}
