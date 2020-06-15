use crate::data::{DamlJsonCreatedEvent, DamlJsonExerciseResult, DamlJsonParty, DamlJsonQuery};
use crate::error::{DamlJsonError, DamlJsonResult};
use crate::request::{
    DamlJsonAllocatePartyRequest, DamlJsonAllocatePartyResponse, DamlJsonCreateAndExerciseRequest,
    DamlJsonCreateAndExerciseResponse, DamlJsonCreateRequest, DamlJsonCreateResponse, DamlJsonErrorResponse,
    DamlJsonExerciseByKeyRequest, DamlJsonExerciseByKeyResponse, DamlJsonExerciseRequest, DamlJsonExerciseResponse,
    DamlJsonFetchByKeyRequest, DamlJsonFetchPartiesRequest, DamlJsonFetchPartiesResponse, DamlJsonFetchRequest,
    DamlJsonFetchResponse, DamlJsonListPackagesResponse, DamlJsonQueryResponse, DamlJsonRequestMeta,
    DamlJsonUploadDarResponse,
};
use bytes::Bytes;
use reqwest::{Certificate, Client, ClientBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use url::Url;

static CREATE_REST: &str = "/v1/create";
static EXERCISE_REST: &str = "/v1/exercise";
static CREATE_AND_EXERCISE_REST: &str = "/v1/create-and-exercise";
static FETCH_REST: &str = "/v1/fetch";
static QUERY_REST: &str = "/v1/query";
static PARTIES_REST: &str = "/v1/parties";
static ALLOCATE_PARTY_REST: &str = "/v1/parties/allocate";
static PACKAGES_REST: &str = "/v1/packages";

const DEFAULT_TIMEOUT_SECS: u64 = 5;

/// DAML JSON client configuration options.
#[derive(Debug, Default)]
pub struct DamlJsonClientConfig {
    url: String,
    connect_timeout: Duration,
    timeout: Duration,
    keepalive: Option<Duration>,
    nodelay: bool,
    max_idle_connection_per_host: usize,
    tls_config: Option<DamlJsonTlsConfig>,
    auth_token: Option<String>,
}

/// DAML JSON client TLS configuration.
#[derive(Debug)]
pub struct DamlJsonTlsConfig {
    ca_cert: Vec<u8>,
}

/// DAML JSON client builder.
pub struct DamlJsonClientBuilder {
    config: DamlJsonClientConfig,
}

impl DamlJsonClientBuilder {
    /// Create a new [`DamlJsonClientBuilder`] for a given `url`.
    pub fn url(url: impl Into<String>) -> Self {
        Self {
            config: DamlJsonClientConfig {
                url: url.into(),
                connect_timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
                ..DamlJsonClientConfig::default()
            },
        }
    }

    /// Set the connection timeout.
    pub fn connect_timeout(self, connect_timeout: Duration) -> Self {
        Self {
            config: DamlJsonClientConfig {
                connect_timeout,
                ..self.config
            },
        }
    }

    /// Set the timeout.
    pub fn timeout(self, timeout: Duration) -> Self {
        Self {
            config: DamlJsonClientConfig {
                timeout,
                ..self.config
            },
        }
    }

    /// Enable TCP keepalive.
    pub fn keepalive(self, keepalive: Duration) -> Self {
        Self {
            config: DamlJsonClientConfig {
                keepalive: Some(keepalive),
                ..self.config
            },
        }
    }

    /// Enable TCP nodelay.
    pub fn nodelay(self) -> Self {
        Self {
            config: DamlJsonClientConfig {
                nodelay: true,
                ..self.config
            },
        }
    }

    /// Set the maximum number of idle connections allowed per host.
    pub fn max_idle_connection_per_host(self, max: usize) -> Self {
        Self {
            config: DamlJsonClientConfig {
                max_idle_connection_per_host: max,
                ..self.config
            },
        }
    }

    /// Set the TLS root CA.
    pub fn with_tls(self, ca_cert: impl Into<Vec<u8>>) -> Self {
        Self {
            config: DamlJsonClientConfig {
                tls_config: Some(DamlJsonTlsConfig {
                    ca_cert: ca_cert.into(),
                }),
                ..self.config
            },
        }
    }

    /// Set the Bearer auth token.
    pub fn with_auth(self, auth_token: String) -> Self {
        Self {
            config: DamlJsonClientConfig {
                auth_token: Some(auth_token),
                ..self.config
            },
        }
    }

    /// Build the [`DamlJsonClient`] from the configuration.
    pub fn build(self) -> DamlJsonResult<DamlJsonClient> {
        DamlJsonClient::new_from_config(self.config)
    }
}

/// DAML JSON API client.
///
/// See [here](https://docs.daml.com/json-api) for full details of the DAML JSON API.
pub struct DamlJsonClient {
    client: Client,
    config: DamlJsonClientConfig,
}

impl DamlJsonClient {
    /// Create a new [`DamlJsonClient`].
    pub fn new(url: impl Into<String>, token: Option<String>) -> DamlJsonResult<Self> {
        Ok(Self {
            client: Client::new(),
            config: DamlJsonClientConfig {
                url: url.into(),
                auth_token: token,
                ..DamlJsonClientConfig::default()
            },
        })
    }

    /// Create a new [`DamlJsonClient`] from a [`JsonClientConfig`].
    pub fn new_from_config(config: DamlJsonClientConfig) -> DamlJsonResult<Self> {
        let mut builder = ClientBuilder::default()
            .connect_timeout(config.connect_timeout)
            .timeout(config.timeout)
            .pool_idle_timeout(config.keepalive)
            .tcp_nodelay_(config.nodelay)
            .pool_max_idle_per_host(config.max_idle_connection_per_host)
            .use_rustls_tls();
        if let Some(cc) = &config.tls_config {
            builder = builder.add_root_certificate(Certificate::from_pem(&cc.ca_cert)?);
        }
        Ok(Self {
            client: builder.build()?,
            config,
        })
    }

    /// Return the current configuration.
    pub const fn config(&self) -> &DamlJsonClientConfig {
        &self.config
    }

    /// Create a new `DAML` contract.
    pub async fn create(&self, template_id: &str, payload: Value) -> DamlJsonResult<DamlJsonCreatedEvent> {
        Ok(self.create_request(&DamlJsonCreateRequest::new(template_id, payload)).await?.result)
    }

    /// Create a new `DAML` Contract with optional meta field.
    pub async fn create_with_meta(
        &self,
        template_id: &str,
        payload: Value,
        meta: DamlJsonRequestMeta,
    ) -> DamlJsonResult<DamlJsonCreatedEvent> {
        Ok(self.create_request(&DamlJsonCreateRequest::new_with_meta(template_id, payload, meta)).await?.result)
    }

    /// Exercise a `DAML` choice by contract id.
    pub async fn exercise(
        &self,
        template_id: &str,
        contract_id: &str,
        choice: &str,
        argument: Value,
    ) -> DamlJsonResult<DamlJsonExerciseResult> {
        Ok(self
            .exercise_request(&DamlJsonExerciseRequest::new(template_id, contract_id, choice, argument))
            .await?
            .result)
    }

    /// Exercise a `DAML` choice by contract key.
    pub async fn exercise_by_key(
        &self,
        template_id: &str,
        key: Value,
        choice: &str,
        argument: Value,
    ) -> DamlJsonResult<DamlJsonExerciseResult> {
        Ok(self
            .exercise_by_key_request(&DamlJsonExerciseByKeyRequest::new(template_id, key, choice, argument))
            .await?
            .result)
    }

    /// Create and exercise a `DAML` choice.
    pub async fn create_and_exercise(
        &self,
        template_id: &str,
        payload: Value,
        choice: &str,
        argument: Value,
    ) -> DamlJsonResult<DamlJsonExerciseResult> {
        Ok(self
            .create_and_exercise_request(&DamlJsonCreateAndExerciseRequest::new(template_id, payload, choice, argument))
            .await?
            .result)
    }

    /// Fetch a `DAML` contract by id.
    pub async fn fetch(&self, contract_id: &str) -> DamlJsonResult<DamlJsonCreatedEvent> {
        Ok(self.fetch_request(&DamlJsonFetchRequest::new(contract_id)).await?.result)
    }

    /// Fetch a `DAML` contract by key.
    pub async fn fetch_by_key(&self, template_id: &str, key: Value) -> DamlJsonResult<DamlJsonCreatedEvent> {
        Ok(self.fetch_by_key_request(&DamlJsonFetchByKeyRequest::new(template_id, key)).await?.result)
    }

    /// List all currently active contracts for all known templates.
    pub async fn query_all(&self) -> DamlJsonResult<Vec<DamlJsonCreatedEvent>> {
        Ok(self.query_all_request().await?.result)
    }

    /// List currently active contracts that match a given query.
    pub async fn query<S: Into<String>>(
        &self,
        template_ids: Vec<S>,
        query: Value,
    ) -> DamlJsonResult<Vec<DamlJsonCreatedEvent>> {
        let templates: Vec<_> = template_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        Ok(self.query_request(&DamlJsonQuery::new(templates, query)).await?.result)
    }

    /// Fetch `DAML` Parties by identifiers.
    ///
    /// Retrieve the [`DamlJsonParty`] entries for the given `parties` identifiers.  Unknown parties are silently
    /// discarded.
    pub async fn fetch_parties<S: Into<String>>(&self, parties: Vec<S>) -> DamlJsonResult<Vec<DamlJsonParty>> {
        Ok(self
            .fetch_parties_request(&DamlJsonFetchPartiesRequest::new(parties.into_iter().map(Into::into).collect()))
            .await?
            .result)
    }

    /// Fetch `DAML` Parties and unknown `DAML` Parties by identifiers.
    ///
    /// Retrieve the [`DamlJsonParty`] entries for the given `parties` identifiers and unknown party identifiers.
    pub async fn fetch_parties_with_unknown<S: Into<String>>(
        &self,
        parties: Vec<S>,
    ) -> DamlJsonResult<(Vec<DamlJsonParty>, Vec<String>)> {
        let response = self
            .fetch_parties_request(&DamlJsonFetchPartiesRequest::new(parties.into_iter().map(Into::into).collect()))
            .await?;
        let unknown_parties =
            response.warnings.and_then(|mut warnings| warnings.remove("unknownParties")).unwrap_or_default();
        Ok((response.result, unknown_parties))
    }

    /// Fetch all known Parties.
    pub async fn fetch_all_parties(&self) -> DamlJsonResult<Vec<DamlJsonParty>> {
        Ok(self.fetch_all_parties_request().await?.result)
    }

    /// Allocate Party.
    pub async fn allocate_party(
        &self,
        identifier_hint: Option<&str>,
        display_name: Option<&str>,
    ) -> DamlJsonResult<DamlJsonParty> {
        Ok(self.allocate_party_request(&DamlJsonAllocatePartyRequest::new(identifier_hint, display_name)).await?.result)
    }

    /// List All `DALF` packages
    pub async fn list_packages(&self) -> DamlJsonResult<Vec<String>> {
        Ok(self.list_packages_request().await?.result)
    }

    /// Download a `DALF` package.
    pub async fn download_package(&self, package_id: &str) -> DamlJsonResult<Vec<u8>> {
        self.download_package_request(package_id).await
    }

    /// Upload a `DAR` file.
    pub async fn upload_dar(&self, content: Vec<u8>) -> DamlJsonResult<()> {
        self.upload_dar_request(content).await?;
        Ok(())
    }

    async fn create_request(&self, request: &DamlJsonCreateRequest) -> DamlJsonResult<DamlJsonCreateResponse> {
        self.post_json(Self::url(&self.config.url, CREATE_REST)?, request).await
    }

    async fn exercise_request(&self, request: &DamlJsonExerciseRequest) -> DamlJsonResult<DamlJsonExerciseResponse> {
        self.post_json(Self::url(&self.config.url, EXERCISE_REST)?, request).await
    }

    async fn exercise_by_key_request(
        &self,
        request: &DamlJsonExerciseByKeyRequest,
    ) -> DamlJsonResult<DamlJsonExerciseByKeyResponse> {
        self.post_json(Self::url(&self.config.url, EXERCISE_REST)?, request).await
    }

    async fn create_and_exercise_request(
        &self,
        request: &DamlJsonCreateAndExerciseRequest,
    ) -> DamlJsonResult<DamlJsonCreateAndExerciseResponse> {
        self.post_json(Self::url(&self.config.url, CREATE_AND_EXERCISE_REST)?, request).await
    }

    async fn fetch_request(&self, request: &DamlJsonFetchRequest) -> DamlJsonResult<DamlJsonFetchResponse> {
        self.post_json(Self::url(&self.config.url, FETCH_REST)?, request).await
    }

    async fn fetch_by_key_request(&self, request: &DamlJsonFetchByKeyRequest) -> DamlJsonResult<DamlJsonFetchResponse> {
        self.post_json(Self::url(&self.config.url, FETCH_REST)?, request).await
    }

    async fn query_all_request(&self) -> DamlJsonResult<DamlJsonQueryResponse> {
        self.get_json(Self::url(&self.config.url, QUERY_REST)?).await
    }

    async fn query_request(&self, request: &DamlJsonQuery) -> DamlJsonResult<DamlJsonQueryResponse> {
        self.post_json(Self::url(&self.config.url, QUERY_REST)?, request).await
    }

    async fn fetch_parties_request(
        &self,
        request: &DamlJsonFetchPartiesRequest,
    ) -> DamlJsonResult<DamlJsonFetchPartiesResponse> {
        self.post_json(Self::url(&self.config.url, PARTIES_REST)?, request).await
    }

    async fn fetch_all_parties_request(&self) -> DamlJsonResult<DamlJsonFetchPartiesResponse> {
        self.get_json(Self::url(&self.config.url, PARTIES_REST)?).await
    }

    async fn allocate_party_request(
        &self,
        request: &DamlJsonAllocatePartyRequest,
    ) -> DamlJsonResult<DamlJsonAllocatePartyResponse> {
        self.post_json(Self::url(&self.config.url, ALLOCATE_PARTY_REST)?, request).await
    }

    async fn list_packages_request(&self) -> DamlJsonResult<DamlJsonListPackagesResponse> {
        self.get_json(Self::url(&self.config.url, PACKAGES_REST)?).await
    }

    async fn download_package_request(&self, package_id: &str) -> DamlJsonResult<Vec<u8>> {
        Ok(self.get_bytes(Self::url(&self.config.url, &format!("{}/{}", PACKAGES_REST, package_id))?).await?.to_vec())
    }

    async fn upload_dar_request(&self, bytes: Vec<u8>) -> DamlJsonResult<DamlJsonUploadDarResponse> {
        Ok(self.post_bytes(Self::url(&self.config.url, PACKAGES_REST)?, bytes).await?)
    }

    async fn get_json<R: DeserializeOwned>(&self, url: Url) -> DamlJsonResult<R> {
        let request = match &self.config.auth_token {
            Some(token) => self.client.get(url).bearer_auth(token),
            None => self.client.get(url),
        };
        self.process_json_response(request.send().await?).await
    }

    async fn get_bytes(&self, url: Url) -> DamlJsonResult<Bytes> {
        let request = match &self.config.auth_token {
            Some(token) => self.client.get(url).bearer_auth(token),
            None => self.client.get(url),
        };
        self.process_bytes_response(request.send().await?).await
    }

    async fn post_json<T: Serialize, R: DeserializeOwned>(&self, url: Url, json: T) -> DamlJsonResult<R> {
        let request = match &self.config.auth_token {
            Some(token) => self.client.post(url).bearer_auth(token),
            None => self.client.post(url),
        };
        self.process_json_response(request.json(&json).send().await?).await
    }

    async fn post_bytes<R: DeserializeOwned>(&self, url: Url, bytes: impl Into<Bytes>) -> DamlJsonResult<R> {
        let request = match &self.config.auth_token {
            Some(token) => self.client.post(url).bearer_auth(token),
            None => self.client.post(url),
        };
        self.process_json_response(request.body(bytes.into()).send().await?).await
    }

    async fn process_json_response<R: DeserializeOwned>(&self, res: Response) -> DamlJsonResult<R> {
        if res.status().is_success() {
            Ok(res.json::<R>().await?)
        } else {
            Err(self.process_error_response(res).await?)
        }
    }

    async fn process_bytes_response(&self, res: Response) -> DamlJsonResult<Bytes> {
        if res.status().is_success() {
            Ok(res.bytes().await?)
        } else {
            Err(self.process_error_response(res).await?)
        }
    }

    async fn process_error_response(&self, error_response: Response) -> DamlJsonResult<DamlJsonError> {
        if error_response.status().is_client_error() || error_response.status().is_server_error() {
            let error_response = error_response.json::<DamlJsonErrorResponse>().await?;
            Ok(DamlJsonError::ErrorResponse(error_response.status, error_response.errors.join(",")))
        } else {
            Ok(DamlJsonError::UnhandledHttpResponse(error_response.status().to_string()))
        }
    }

    fn url(base: &str, path: &str) -> DamlJsonResult<Url> {
        Ok(Url::parse(base)?.join(path)?)
    }
}
