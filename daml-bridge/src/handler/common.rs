use daml_json::request::DamlJsonErrorResponse;
use daml_util::DamlSandboxAuthToken;
use std::error::Error;

pub type JsonResult<T> = Result<T, DamlJsonErrorResponse>;

// TODO tidy up, err and str variants of functions

pub fn internal_server_error(err: impl Error) -> DamlJsonErrorResponse {
    make_error_response(err.to_string(), 500) // TODO constants
}

pub fn bad_request(err: impl Error) -> DamlJsonErrorResponse {
    make_error_response(err.to_string(), 400)
}

pub fn unauthorized(err: impl Error) -> DamlJsonErrorResponse {
    auth_err(err.to_string())
}

pub fn auth_err<S: Into<String>>(err: S) -> DamlJsonErrorResponse {
    make_error_response(err, 401)
}

pub fn make_error_response<S: Into<String>>(err: S, status_code: u16) -> DamlJsonErrorResponse {
    DamlJsonErrorResponse::single(status_code, err.into())
}

/// Parse the JWT token value of the `Authorization` http header into a [`DamlSandboxAuthToken`]
///
/// The header value is expected to have the form `Bearer x.y.z` where `x.y.z` is a JWT token.
pub fn parse_auth_header(jwt_token: Option<&str>) -> JsonResult<(&str, DamlSandboxAuthToken)> {
    jwt_token.ok_or_else(|| auth_err("missing Authorization header with OAuth 2.0 Bearer Token")).and_then(|token| {
        token
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| auth_err("malformed Bearer token in Authorization header"))
            .and_then(|token| Ok((token, DamlSandboxAuthToken::parse_jwt_no_validation(token).map_err(unauthorized)?)))
    })
}

/// Extract the acting party, ledger id and application id from a [`DamlSandboxAuthToken`]
pub fn extract_from_token(parsed_token: &DamlSandboxAuthToken) -> JsonResult<(&str, &str, &str)> {
    let acting_party = parsed_token
        .single_party()
        .ok_or_else(|| auth_err("token must contain exactly one Party (actAs or readAs)"))?;
    let ledger_id = parsed_token.ledger_id().ok_or_else(|| auth_err("token must contain a ledgerId"))?;
    let application_id = parsed_token.application_id().ok_or_else(|| auth_err("token must contain applicationId"))?;
    Ok((acting_party, ledger_id, application_id))
}
