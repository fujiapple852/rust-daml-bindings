use crate::data::DamlError;
use crate::data::DamlResult;

use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::testing::time_service_client::TimeServiceClient;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::testing::{GetTimeRequest, SetTimeRequest};
use crate::util;
use crate::util::Required;
use chrono::DateTime;
use chrono::Utc;
use futures::stream::StreamExt;
use futures::Stream;

use tonic::transport::Channel;
use tonic::Request;

/// Get and set the time of a DAML ledger (requires `testing` feature).
pub struct DamlTimeService {
    channel: Channel,
    ledger_id: String,
}

impl DamlTimeService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
        }
    }

    /// DOCME fully document this
    pub async fn get_time(&self) -> DamlResult<impl Stream<Item = DamlResult<DateTime<Utc>>>> {
        let request = Request::new(GetTimeRequest {
            ledger_id: self.ledger_id.clone(),
        });
        let time_stream = self.client().get_time(request).await?.into_inner();
        Ok(time_stream.map(|item| match item {
            Ok(r) => Ok(util::make_datetime(&r.current_time.req()?)),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    /// DOCME fully document this
    pub async fn set_time(
        &self,
        current_time: impl Into<DateTime<Utc>>,
        new_time: impl Into<DateTime<Utc>>,
    ) -> DamlResult<()> {
        let request = Request::new(SetTimeRequest {
            ledger_id: self.ledger_id.clone(),
            current_time: Some(util::make_timestamp_secs(current_time.into())),
            new_time: Some(util::make_timestamp_secs(new_time.into())),
        });
        self.client().set_time(request).await?;
        Ok(())
    }

    fn client(&self) -> TimeServiceClient<Channel> {
        TimeServiceClient::new(self.channel.clone())
    }
}
