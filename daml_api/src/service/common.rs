use crate::data::{DamlError, DamlResult};
use tonic::metadata::MetadataValue;
use tonic::Request;

pub(crate) fn make_request<T>(payload: T, auth_token: &Option<String>) -> DamlResult<Request<T>> {
    let mut request = Request::new(payload);
    if let Some(token) = auth_token {
        let token =
            MetadataValue::from_str(&format!("Bearer {}", token)).map_err(|e| DamlError::Other(e.to_string()))?;
        request.metadata_mut().insert("authorization", token);
    }
    Ok(request)
}
