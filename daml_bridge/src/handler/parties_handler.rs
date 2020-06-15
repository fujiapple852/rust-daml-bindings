use crate::aliases::{BridgeConfig, GrpcClient};
use crate::handler::common::{internal_server_error, parse_auth_header, JsonResult};
use daml_grpc::data::party::DamlPartyDetails;
use daml_json::data::DamlJsonParty;
use daml_json::request::{
    make_single_warning, DamlJsonAllocatePartyRequest, DamlJsonAllocatePartyResponse, DamlJsonFetchPartiesRequest,
    DamlJsonFetchPartiesResponse,
};

/// DOCME
pub struct PartiesHandler {
    _config: BridgeConfig,
    client: GrpcClient,
}

impl PartiesHandler {
    pub fn new(_config: BridgeConfig, client: GrpcClient) -> Self {
        Self {
            _config,
            client,
        }
    }

    /// DOCME
    pub async fn fetch_parties(
        &self,
        fetch_request: DamlJsonFetchPartiesRequest,
        auth_header: Option<&str>,
    ) -> JsonResult<DamlJsonFetchPartiesResponse> {
        let expected = fetch_request.0.clone();
        let parties = self.execute_fetch(fetch_request, auth_header).await?;
        Ok(make_fetch_response(&parties, &expected))
    }

    /// DOCME
    pub async fn fetch_all_parties(&self, auth_header: Option<&str>) -> JsonResult<DamlJsonFetchPartiesResponse> {
        let parties = self.execute_fetch_all(auth_header).await?;
        Ok(make_fetch_response(&parties, &[]))
    }

    /// DOCME
    pub async fn allocate_parties(
        &self,
        allocate_request: DamlJsonAllocatePartyRequest,
        auth_header: Option<&str>,
    ) -> JsonResult<DamlJsonAllocatePartyResponse> {
        let parties = self.execute_allocate(allocate_request, auth_header).await?;
        Ok(make_allocate_response(&parties))
    }

    async fn execute_fetch(
        &self,
        fetch_request: DamlJsonFetchPartiesRequest,
        auth_header: Option<&str>,
    ) -> JsonResult<Vec<DamlPartyDetails>> {
        let (token, _) = parse_auth_header(auth_header)?;
        self.client
            .party_management_service()
            .with_token(token)
            .get_parties(fetch_request.0)
            .await
            .map_err(internal_server_error)
    }

    async fn execute_fetch_all(&self, auth_header: Option<&str>) -> JsonResult<Vec<DamlPartyDetails>> {
        let (token, _) = parse_auth_header(auth_header)?;
        self.client
            .party_management_service()
            .with_token(token)
            .list_known_parties()
            .await
            .map_err(internal_server_error)
    }

    async fn execute_allocate(
        &self,
        allocate_request: DamlJsonAllocatePartyRequest,
        auth_header: Option<&str>,
    ) -> JsonResult<DamlPartyDetails> {
        let (token, _) = parse_auth_header(auth_header)?;
        self.client
            .party_management_service()
            .with_token(token)
            .allocate_party(
                allocate_request.identifier_hint.unwrap_or_default(),
                allocate_request.display_name.unwrap_or_default(),
            )
            .await
            .map_err(internal_server_error)
    }
}

fn make_fetch_response(parties: &[DamlPartyDetails], expected: &[String]) -> DamlJsonFetchPartiesResponse {
    let missing_parties = find_missing_parties(parties, expected);
    let warnings = if missing_parties.is_empty() {
        None
    } else {
        Some(make_single_warning("unknownParties", missing_parties))
    };
    DamlJsonFetchPartiesResponse {
        status: 200,
        result: parties.iter().map(to_json_party).collect(),
        warnings,
    }
}

fn make_allocate_response(allocated_party: &DamlPartyDetails) -> DamlJsonAllocatePartyResponse {
    DamlJsonAllocatePartyResponse {
        status: 200,
        result: to_json_party(allocated_party),
        warnings: None,
    }
}

fn find_missing_parties(parties: &[DamlPartyDetails], expected: &[String]) -> Vec<String> {
    expected
        .iter()
        .filter_map(|p| {
            if parties.iter().all(|pd| pd.party() != *p) {
                Some(String::from(p))
            } else {
                None
            }
        })
        .collect()
}

fn to_json_party(party: &DamlPartyDetails) -> DamlJsonParty {
    DamlJsonParty::new(party.party(), party.display_name(), party.is_local())
}
