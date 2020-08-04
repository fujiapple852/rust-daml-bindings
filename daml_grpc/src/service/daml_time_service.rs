use crate::data::DamlError;
use crate::data::DamlResult;
use crate::grpc_protobuf::com::daml::ledger::api::v1::testing::time_service_client::TimeServiceClient;
use crate::grpc_protobuf::com::daml::ledger::api::v1::testing::{GetTimeRequest, SetTimeRequest};
use crate::service::common::make_request;
use crate::util;
use crate::util::Required;
use chrono::DateTime;
use chrono::Utc;
use futures::stream::StreamExt;
use futures::Stream;
use log::{debug, trace};
use tonic::transport::Channel;

/// Get and set the time of a DAML ledger (requires `testing` feature).
pub struct DamlTimeService<'a> {
    channel: Channel,
    ledger_id: &'a str,
    auth_token: Option<&'a str>,
}

impl<'a> DamlTimeService<'a> {
    pub fn new(channel: Channel, ledger_id: &'a str, auth_token: Option<&'a str>) -> Self {
        Self {
            channel,
            ledger_id,
            auth_token,
        }
    }

    /// Override the JWT token to use for this service.
    pub fn with_token(self, auth_token: &'a str) -> Self {
        Self {
            auth_token: Some(auth_token),
            ..self
        }
    }

    /// Override the ledger id to use for this service.
    pub fn with_ledger_id(self, ledger_id: &'a str) -> Self {
        Self {
            ledger_id,
            ..self
        }
    }

    /// DOCME fully document this
    pub async fn get_time(&self) -> DamlResult<impl Stream<Item = DamlResult<DateTime<Utc>>>> {
        debug!("get_time");
        let payload = GetTimeRequest {
            ledger_id: self.ledger_id.to_string(),
        };
        trace!("get_time payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        let time_stream =
            self.client().get_time(make_request(payload, self.auth_token.as_deref())?).await?.into_inner();
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
            ledger_id: self.ledger_id.to_string(),
            current_time: Some(util::to_grpc_timestamp(current_time.into())?),
            new_time: Some(util::to_grpc_timestamp(new_time.into())?),
        };
        trace!("set_time payload = {:?}, auth_token = {:?}", payload, self.auth_token);
        self.client().set_time(make_request(payload, self.auth_token.as_deref())?).await?;
        Ok(())
    }

    fn client(&self) -> TimeServiceClient<Channel> {
        TimeServiceClient::new(self.channel.clone())
    }
}
