use crate::data::DamlError;
use crate::data::DamlResult;

use crate::grpc_protobuf::com::daml::ledger::api::v1::testing::time_service_client::TimeServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::testing::{GetTimeRequest, SetTimeRequest};
use crate::util;
use crate::util::Required;
use chrono::DateTime;
use chrono::Utc;
use futures::stream::StreamExt;
use futures::Stream;

use crate::ledger_client::DamlTokenRefresh;
use crate::service::common::make_request;
use log::{debug, trace};
use tonic::transport::Channel;

/// Get and set the time of a DAML ledger (requires `testing` feature).
pub struct DamlTimeService {
    channel: Channel,
    ledger_id: String,
    auth_token: Option<String>,
}

impl DamlTimeService {
    pub fn new(channel: Channel, ledger_id: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            channel,
            ledger_id: ledger_id.into(),
            auth_token,
        }
    }

    /// DOCME fully document this
    pub async fn get_time(&self) -> DamlResult<impl Stream<Item = DamlResult<DateTime<Utc>>>> {
        debug!("get_time");
        let payload = GetTimeRequest {
            ledger_id: self.ledger_id.clone(),
        };
        trace!("get_time payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let time_stream = self.client().get_time(make_request(payload, &self.auth_token)?).await?.into_inner();
        Ok(time_stream.map(|item| match item {
            Ok(r) => Ok(util::from_grpc_timestamp(&r.current_time.req()?)),
            Err(e) => Err(DamlError::from(e)),
        }))
    }

    /// DOCME fully document this
    pub async fn set_time(
        &self,
        current_time: impl Into<DateTime<Utc>>,
        new_time: impl Into<DateTime<Utc>>,
    ) -> DamlResult<()> {
        debug!("set_time");
        let payload = SetTimeRequest {
            ledger_id: self.ledger_id.clone(),
            current_time: Some(util::to_grpc_timestamp(current_time.into())?),
            new_time: Some(util::to_grpc_timestamp(new_time.into())?),
        };
        trace!("set_time payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client().set_time(make_request(payload, &self.auth_token)?).await?;
        Ok(())
    }

    fn client(&self) -> TimeServiceClient<Channel> {
        TimeServiceClient::new(self.channel.clone())
    }
}

impl DamlTokenRefresh for DamlTimeService {
    fn refresh_token(&mut self, auth_token: Option<String>) {
        self.auth_token = auth_token
    }
}
