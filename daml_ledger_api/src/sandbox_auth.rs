use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

use crate::data::{DamlError, DamlResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Build JWT tokens suitable for use in the DAML Sandbox.
///
/// The DAML Sandbox support the use JWT tokens for authentication.  The following JSON structure represents the claims
/// that may be supplied (see [here](https://docs.daml.com/tools/sandbox.html#running-with-authentication) for details):
///
/// ```json
/// {
///   "https://daml.com/ledger-api": {
///     "ledgerId": "my-ledger",
///     "participantId": null,
///     "applicationId": null,
///     "admin": true,
///     "actAs": ["Alice"],
///     "readAs": ["Alice", "Bob"]
///   },
///   "exp": 1300819380,
/// }
/// ```
///
/// All ledger API endpoints support passing a `Bearer` token in the `authentication` http header.  This builder
/// produces bearer token strings in `HS256`, `RS256` & `EC256` formats which are suitable for use by the DAML ledger
/// API.
///
/// Note that test JWT tokens created with [https://jwt.io/](https://jwt.io/) will, by default, place the `alg` attribute ahead of
/// the `typ` attribute in the header whereas the library used here will places them the opposite wa around.  Whilst
/// both produce valid tokens this can be confusing when trying to compare examples.
///
/// # Examples
///
/// A `HS256` (shared secret) bearer token matching the xample above can be created as follows:
///
/// ```
/// # use daml_ledger_api::data::DamlResult;
/// # fn main() -> DamlResult<()> {
/// use daml_ledger_api::DamlSandboxTokenBuilder;
///
/// let token = DamlSandboxTokenBuilder::new_with_expiry(1300819380)
///     .ledger_id("my-ledger")
///     .admin(true)
///     .act_as(vec!["Alice".to_owned()])
///     .read_as(vec!["Alice".to_owned(), "Bob".to_owned()])
///     .new_hs256_unsafe_token("some secret phrase")?;
/// # Ok(())
/// # }
/// ```
///
/// The generated token can then supplied to the [`DamlLedgerClientBuilder`] via the `with_auth` method as follows:
///
/// ```no_run
/// # use daml_ledger_api::data::DamlResult;
/// # #[tokio::main]
/// # async fn main() -> DamlResult<()> {
/// use daml_ledger_api::DamlSandboxTokenBuilder;
/// use daml_ledger_api::DamlLedgerClientBuilder;
///
/// let token = DamlSandboxTokenBuilder::new_with_expiry(1300819380)
///     .ledger_id("my-ledger")
///     .admin(true)
///     .act_as(vec!["Alice".to_owned()])
///     .read_as(vec!["Alice".to_owned(), "Bob".to_owned()])
///     .new_ec256_token("... EC256 key in bytes ...")?;
///
/// let ledger_client = DamlLedgerClientBuilder::uri("http://localhost:8080")
///                         .with_auth(token)
///                         .connect()
///                         .await?;
/// # Ok(())
/// # }
/// ```
///
/// [`DamlLedgerClientBuilder`]: crate::DamlLedgerClientBuilder
#[derive(Default)]
pub struct DamlSandboxTokenBuilder {
    ledger_id: Option<String>,
    participant_id: Option<String>,
    application_id: Option<String>,
    admin: bool,
    act_as: Vec<String>,
    read_as: Vec<String>,
    expiry: i64,
}

impl DamlSandboxTokenBuilder {
    /// Create with an expiry relative to the current system time.
    pub fn new_with_duration_secs(secs: i64) -> Self {
        Self {
            expiry: Utc::now().timestamp() + secs,
            ..DamlSandboxTokenBuilder::default()
        }
    }

    /// Create with an absolute expiry timestamp (unix).
    pub fn new_with_expiry(timestamp: i64) -> Self {
        Self {
            expiry: timestamp,
            ..DamlSandboxTokenBuilder::default()
        }
    }

    pub fn ledger_id(self, ledger_id: impl Into<String>) -> Self {
        Self {
            ledger_id: Some(ledger_id.into()),
            ..self
        }
    }

    pub fn participant_id(self, participant_id: impl Into<String>) -> Self {
        Self {
            participant_id: Some(participant_id.into()),
            ..self
        }
    }

    pub fn application_id(self, application_id: impl Into<String>) -> Self {
        Self {
            application_id: Some(application_id.into()),
            ..self
        }
    }

    pub fn admin(self, admin: bool) -> Self {
        Self {
            admin,
            ..self
        }
    }

    pub fn act_as(self, act_as: Vec<String>) -> Self {
        Self {
            act_as,
            ..self
        }
    }

    pub fn read_as(self, read_as: Vec<String>) -> Self {
        Self {
            read_as,
            ..self
        }
    }

    /// Create a new HS256 JWT token based on a shared secret.
    ///
    /// This approach is considered unsafe for production use and should be used for local testing only.  Note that
    /// whilst the method name contains the word `unsafe` to highlight the above, the method does not contain any
    /// `unsafe` blocks or call any `unsafe` methods.
    pub fn new_hs256_unsafe_token(self, secret: impl AsRef<[u8]>) -> DamlResult<String> {
        let encoding_key = &EncodingKey::from_secret(secret.as_ref());
        self.generate_token(Algorithm::HS256, encoding_key)
    }

    /// Create a new RS256 JWT token based on the supplied RSA key.
    ///
    /// The key is expected to be in `pem` format.
    pub fn new_rs256_token(self, rsa_pem: impl AsRef<[u8]>) -> DamlResult<String> {
        let encoding_key = &EncodingKey::from_rsa_pem(rsa_pem.as_ref()).map_err(|e| DamlError::Other(e.to_string()))?;
        self.generate_token(Algorithm::RS256, encoding_key)
    }

    /// Create a new EC256 JWT token based on the supplied RSA key.
    ///
    /// The key is expected to be in `pem` format.
    pub fn new_ec256_token(self, ec_pem: impl AsRef<[u8]>) -> DamlResult<String> {
        let encoding_key = &EncodingKey::from_ec_pem(ec_pem.as_ref()).map_err(|e| DamlError::Other(e.to_string()))?;
        self.generate_token(Algorithm::ES256, encoding_key)
    }

    fn generate_token(self, algorithm: Algorithm, encoding_key: &EncodingKey) -> DamlResult<String> {
        Ok(encode(&Header::new(algorithm), &self.into_token(), encoding_key)
            .map_err(|e| DamlError::Other(e.to_string()))?)
    }

    fn into_token(self) -> DamlSandboxAuthToken {
        DamlSandboxAuthToken {
            details: DamlSandboxAuthDetails {
                ledger_id: self.ledger_id,
                participant_id: self.participant_id,
                application_id: self.application_id,
                admin: self.admin,
                act_as: self.act_as,
                read_as: self.read_as,
            },
            exp: self.expiry,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DamlSandboxAuthToken {
    #[serde(rename = "https://daml.com/ledger-api")]
    details: DamlSandboxAuthDetails,
    exp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DamlSandboxAuthDetails {
    ledger_id: Option<String>,
    participant_id: Option<String>,
    application_id: Option<String>,
    admin: bool,
    act_as: Vec<String>,
    read_as: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    #[test]
    fn test_serialise() {
        let token = DamlSandboxAuthToken {
            details: DamlSandboxAuthDetails {
                ledger_id: Some("test-sandbox".to_owned()),
                participant_id: None,
                application_id: None,
                admin: true,
                act_as: vec!["Alice".to_owned(), "Bob".to_owned()],
                read_as: vec!["Alice".to_owned(), "Bob".to_owned()],
            },
            exp: 1_581_292_002,
        };
        let serialized = serde_json::to_string(&token).unwrap();
        assert_eq!(
            r#"{"https://daml.com/ledger-api":{"ledgerId":"test-sandbox","participantId":null,"applicationId":null,"admin":true,"actAs":["Alice","Bob"],"readAs":["Alice","Bob"]},"exp":1581292002}"#,
            serialized
        );
    }

    #[test]
    fn test_encode_with_secret() {
        let token = DamlSandboxAuthToken {
            details: DamlSandboxAuthDetails {
                ledger_id: Some("sandbox".to_owned()),
                participant_id: None,
                application_id: None,
                admin: true,
                act_as: vec!["Alice".to_owned(), "Bob".to_owned()],
                read_as: vec!["Alice".to_owned(), "Bob".to_owned()],
            },
            exp: 1_581_292_002,
        };
        let token_str =
            encode(&Header::default(), &token, &EncodingKey::from_secret("testsecret".as_ref())).expect("token");
        assert_eq!(
            r#"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJodHRwczovL2RhbWwuY29tL2xlZGdlci1hcGkiOnsibGVkZ2VySWQiOiJzYW5kYm94IiwicGFydGljaXBhbnRJZCI6bnVsbCwiYXBwbGljYXRpb25JZCI6bnVsbCwiYWRtaW4iOnRydWUsImFjdEFzIjpbIkFsaWNlIiwiQm9iIl0sInJlYWRBcyI6WyJBbGljZSIsIkJvYiJdfSwiZXhwIjoxNTgxMjkyMDAyfQ.Y-3GYosItlnhTXOTgwE_TjP_D_Q0Pvw-pqe20OTwnIg"#,
            token_str
        );
    }

    #[test]
    fn test_builder_with_secret() -> DamlResult<()> {
        let token_str = DamlSandboxTokenBuilder::new_with_expiry(1_581_292_002)
            .ledger_id("sandbox")
            .admin(true)
            .act_as(vec!["Alice".to_owned(), "Bob".to_owned()])
            .read_as(vec!["Alice".to_owned(), "Bob".to_owned()])
            .new_hs256_unsafe_token("testsecret")?;
        assert_eq!(
            r#"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJodHRwczovL2RhbWwuY29tL2xlZGdlci1hcGkiOnsibGVkZ2VySWQiOiJzYW5kYm94IiwicGFydGljaXBhbnRJZCI6bnVsbCwiYXBwbGljYXRpb25JZCI6bnVsbCwiYWRtaW4iOnRydWUsImFjdEFzIjpbIkFsaWNlIiwiQm9iIl0sInJlYWRBcyI6WyJBbGljZSIsIkJvYiJdfSwiZXhwIjoxNTgxMjkyMDAyfQ.Y-3GYosItlnhTXOTgwE_TjP_D_Q0Pvw-pqe20OTwnIg"#,
            token_str
        );
        Ok(())
    }
}
