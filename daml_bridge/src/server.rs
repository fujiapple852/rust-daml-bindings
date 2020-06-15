use crate::aliases::{Archive, BridgeConfig, GrpcClient};
use crate::handler::create_and_exercise_handler::CreateAndExerciseHandler;
use crate::handler::create_handler::CreateHandler;
use crate::handler::exercise_by_key_handler::ExerciseByKeyHandler;
use crate::handler::exercise_handler::ExerciseHandler;
use crate::handler::packages_handler::PackagesHandler;
use crate::handler::parties_handler::PartiesHandler;
use daml_json::request::{
    DamlJsonAllocatePartyRequest, DamlJsonCreateAndExerciseRequest, DamlJsonCreateRequest, DamlJsonErrorResponse,
    DamlJsonExerciseRequestType, DamlJsonFetchPartiesRequest,
};
use futures::Future;
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use warp::http::StatusCode;
use warp::hyper::body::Bytes;
use warp::path::FullPath;
use warp::reply::{json, with_status, WithStatus};
use warp::Filter;

/// Make the server.
///
/// Create the async http server which will process all incoming JSON API requests.
///
/// Implementation notes:
///
/// - This server uses [`warp`](https://docs.rs/warp/) which is build on top of [`hyper`](https://hyper.rs/) which in
/// turn uses [`tokio`](https://tokio.rs/).
/// - All warp specific code lives in this module and the handlers for the various REST endpoints are delegated to the
/// [`handler`] module which is agnostic to the http server or the driving async runtime.
pub fn make_server(
    config: BridgeConfig,
    archive: Archive,
    grpc_client: GrpcClient,
) -> anyhow::Result<impl Future<Output = ()>> {
    let address = format!("{}:{}", config.http_host(), config.http_port());
    let api = make_api(config, archive, grpc_client);
    Ok(warp::serve(api).run(address.parse::<SocketAddr>()?))
}

fn make_api(
    config: BridgeConfig,
    archive: Archive,
    grpc_client: GrpcClient,
) -> impl Filter<Extract = impl warp::Reply, Error = Infallible> + Clone {
    make_create_filter(config.clone(), archive.clone(), grpc_client.clone())
        .or(make_exercise_filter(config.clone(), archive.clone(), grpc_client.clone()))
        .or(make_create_and_exercise_filter(config.clone(), archive, grpc_client.clone()))
        .or(make_fetch_parties_filter(config.clone(), grpc_client.clone()))
        .or(make_fetch_all_parties_filter(config.clone(), grpc_client.clone()))
        .or(make_parties_allocate_filter(config.clone(), grpc_client.clone()))
        .or(make_list_all_packages_filter(config.clone(), grpc_client.clone()))
        .or(make_get_package_filter(config.clone(), grpc_client.clone()))
        .or(make_upload_dar_filter(config, grpc_client))
        .or(make_unknown_filter())
}

/// POST /v1/create
fn make_create_filter(
    config: BridgeConfig,
    archive: Archive,
    grpc_client: GrpcClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("v1" / "create")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::header::optional("Authorization"))
        .and(with_config(config))
        .and(with_archive(archive))
        .and(with_grpc(grpc_client))
        .and_then(create_handler)
}

/// POST /v1/exercise
fn make_exercise_filter(
    config: BridgeConfig,
    archive: Archive,
    grpc_client: GrpcClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("v1" / "exercise")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::header::optional("Authorization"))
        .and(with_config(config))
        .and(with_archive(archive))
        .and(with_grpc(grpc_client))
        .and_then(exercise_handler)
}

/// POST /v1/create-and-exercise
fn make_create_and_exercise_filter(
    config: BridgeConfig,
    archive: Archive,
    grpc_client: GrpcClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("v1" / "create-and-exercise")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::header::optional("Authorization"))
        .and(with_config(config))
        .and(with_archive(archive))
        .and(with_grpc(grpc_client))
        .and_then(create_and_exercise_handler)
}

/// POST /v1/parties
fn make_fetch_parties_filter(
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("v1" / "parties")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::header::optional("Authorization"))
        .and(with_config(config))
        .and(with_grpc(grpc_client))
        .and_then(fetch_parties_handler)
}

/// GET /v1/parties
fn make_fetch_all_parties_filter(
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("v1" / "parties")
        .and(warp::get())
        .and(warp::header::optional("Authorization"))
        .and(with_config(config))
        .and(with_grpc(grpc_client))
        .and_then(fetch_all_parties_handler)
}

/// POST /v1/parties/allocate
fn make_parties_allocate_filter(
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("v1" / "parties" / "allocate")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::header::optional("Authorization"))
        .and(with_config(config))
        .and(with_grpc(grpc_client))
        .and_then(allocate_party_handler)
}

/// GET /v1/packages
fn make_list_all_packages_filter(
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("v1" / "packages")
        .and(warp::get())
        .and(warp::header::optional("Authorization"))
        .and(with_config(config))
        .and(with_grpc(grpc_client))
        .and_then(list_all_packages_handler)
}

/// GET /v1/packages/:packageid
fn make_get_package_filter(
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("v1" / "packages" / String)
        .and(warp::get())
        .and(warp::header::optional("Authorization"))
        .and(with_config(config))
        .and(with_grpc(grpc_client))
        .and_then(get_package_handler)
        .with(warp::reply::with::header("Transfer-Encoding", "chunked"))
}

/// POST /v1/packages
fn make_upload_dar_filter(
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("v1" / "packages")
        .and(warp::post())
        .and(warp::body::bytes())
        .and(warp::header::exact_ignore_case("Content-Type", "application/octet-stream"))
        .and(warp::header::optional("Authorization"))
        .and(with_config(config))
        .and(with_grpc(grpc_client))
        .and_then(upload_dar_handler)
}

/// Catch all filter to provide a well formed not found error response
fn make_unknown_filter() -> impl Filter<Extract = impl warp::Reply, Error = Infallible> + Clone {
    warp::any().and(warp::path::full().map(unknown_handler))
}

fn with_config(
    config: BridgeConfig,
) -> impl Filter<Extract = (BridgeConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

fn with_archive(archive: Archive) -> impl Filter<Extract = (Archive,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || archive.clone())
}

fn with_grpc(grpc: GrpcClient) -> impl Filter<Extract = (GrpcClient,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || grpc.clone())
}

async fn create_handler(
    create_request: DamlJsonCreateRequest,
    jwt_token: Option<String>,
    config: BridgeConfig,
    archive: Archive,
    grpc_client: GrpcClient,
) -> Result<impl warp::Reply, Infallible> {
    Ok(match CreateHandler::new(config, archive, grpc_client).create(create_request, jwt_token.as_deref()).await {
        Ok(response) => ok_response(&response),
        Err(error) => err_response(&error),
    })
}

async fn exercise_handler(
    exercise_request: DamlJsonExerciseRequestType,
    jwt_token: Option<String>,
    config: BridgeConfig,
    archive: Archive,
    grpc_client: GrpcClient,
) -> Result<impl warp::Reply, Infallible> {
    Ok(match exercise_request {
        DamlJsonExerciseRequestType::Exercise(req) =>
            match ExerciseHandler::new(config, archive, grpc_client).exercise(req, jwt_token.as_deref()).await {
                Ok(response) => ok_response(&response),
                Err(error) => err_response(&error),
            },
        DamlJsonExerciseRequestType::ExerciseByKey(req) =>
            match ExerciseByKeyHandler::new(config, archive, grpc_client)
                .exercise_by_key(req, jwt_token.as_deref())
                .await
            {
                Ok(response) => ok_response(&response),
                Err(error) => err_response(&error),
            },
        DamlJsonExerciseRequestType::Invalid(_) => err_response(&DamlJsonErrorResponse::single(
            400,
            "key and contractId fields are mutually exclusive".to_owned(),
        )),
    })
}

async fn create_and_exercise_handler(
    create_and_exercise_request: DamlJsonCreateAndExerciseRequest,
    jwt_token: Option<String>,
    config: BridgeConfig,
    archive: Archive,
    grpc_client: GrpcClient,
) -> Result<impl warp::Reply, Infallible> {
    Ok(
        match CreateAndExerciseHandler::new(config, archive, grpc_client)
            .create_and_exercise(create_and_exercise_request, jwt_token.as_deref())
            .await
        {
            Ok(response) => ok_response(&response),
            Err(error) => err_response(&error),
        },
    )
}

async fn fetch_parties_handler(
    fetch_request: DamlJsonFetchPartiesRequest,
    jwt_token: Option<String>,
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> Result<impl warp::Reply, Infallible> {
    Ok(match PartiesHandler::new(config, grpc_client).fetch_parties(fetch_request, jwt_token.as_deref()).await {
        Ok(response) => ok_response(&response),
        Err(error) => err_response(&error),
    })
}

async fn fetch_all_parties_handler(
    jwt_token: Option<String>,
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> Result<impl warp::Reply, Infallible> {
    Ok(match PartiesHandler::new(config, grpc_client).fetch_all_parties(jwt_token.as_deref()).await {
        Ok(response) => ok_response(&response),
        Err(error) => err_response(&error),
    })
}

async fn allocate_party_handler(
    allocate_request: DamlJsonAllocatePartyRequest,
    jwt_token: Option<String>,
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> Result<impl warp::Reply, Infallible> {
    Ok(match PartiesHandler::new(config, grpc_client).allocate_parties(allocate_request, jwt_token.as_deref()).await {
        Ok(response) => ok_response(&response),
        Err(error) => err_response(&error),
    })
}

async fn list_all_packages_handler(
    jwt_token: Option<String>,
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> Result<impl warp::Reply, Infallible> {
    Ok(match PackagesHandler::new(config, grpc_client).list_all_packages(jwt_token.as_deref()).await {
        Ok(response) => ok_response(&response),
        Err(error) => err_response(&error),
    })
}

async fn get_package_handler(
    package_id: String,
    jwt_token: Option<String>,
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    Ok(match PackagesHandler::new(config, grpc_client).get_package(&package_id, jwt_token.as_deref()).await {
        Ok(response) => Box::new(ok_response_bytes(response)),
        Err(error) => Box::new(err_response(&error)),
    })
}

async fn upload_dar_handler(
    payload: Bytes,
    jwt_token: Option<String>,
    config: BridgeConfig,
    grpc_client: GrpcClient,
) -> Result<impl warp::Reply, Infallible> {
    Ok(match PackagesHandler::new(config, grpc_client).upload_dar(payload, jwt_token.as_deref()).await {
        Ok(response) => ok_response(&response),
        Err(error) => err_response(&error),
    })
}

fn unknown_handler(path: FullPath) -> impl warp::Reply {
    err_response(&DamlJsonErrorResponse::single(404, format!("not found: {}", path.as_str())))
}

fn ok_response<T: Serialize>(val: &T) -> WithStatus<warp::reply::Json> {
    with_status(json(val), StatusCode::OK)
}

fn ok_response_bytes(bytes: Vec<u8>) -> WithStatus<Vec<u8>> {
    with_status(bytes, StatusCode::OK)
}

fn err_response(error: &DamlJsonErrorResponse) -> WithStatus<warp::reply::Json> {
    with_status(json(error), StatusCode::from_u16(error.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
}
