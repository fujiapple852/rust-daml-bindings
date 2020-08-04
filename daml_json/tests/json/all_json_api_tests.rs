#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::similar_names, clippy::missing_errors_doc, clippy::used_underscore_binding)]
use daml::util::DamlSandboxTokenBuilder;
use daml_grpc::DamlGrpcClientBuilder;
use daml_json::data::{DamlJsonCreatedEvent, DamlJsonEvent, DamlJsonParty};
use daml_json::request::DamlJsonRequestMeta;
use daml_json::service::{DamlJsonClient, DamlJsonClientBuilder};
use daml_lf::DarFile;
use serde_json::json;
use std::io::Read;
use tokio::sync::Mutex;
use tokio::time::Duration;

pub const ALICE_PARTY: &str = "Alice";
pub const SANDBOX_REST_URL: &str = "http://127.0.0.1:7575";
pub const SANDBOX_GRPC_URL: &str = "http://127.0.0.1:8085";
pub const CONNECT_TIMEOUT_MS: u64 = 20000;
pub const TOKEN_KEY_PATH: &str = "../resources/testing_types_sandbox/.auth_certs/es256.key";

#[tokio::test]
async fn test_create() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let create_response =
        alice_client.create("DA.PingPong:Ping", json!({ "sender": "Alice", "receiver": "Bob", "count": 0 })).await?;
    assert_eq!(create_response.payload, json!({ "sender": "Alice", "receiver": "Bob", "count": "0" }));
    Ok(())
}

#[tokio::test]
async fn test_create_with_meta() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let create_response = alice_client
        .create_with_meta(
            "DA.PingPong:Ping",
            json!({ "sender": "Alice", "receiver": "Bob", "count": 0 }),
            DamlJsonRequestMeta::new("cmd-1234"),
        )
        .await?;
    assert_eq!(create_response.payload, json!({ "sender": "Alice", "receiver": "Bob", "count": "0" }));
    Ok(())
}

#[tokio::test]
async fn test_exercise() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let create_response =
        alice_client.create("DA.PingPong:Ping", json!({ "sender": "Alice", "receiver": "Bob", "count": 0 })).await?;
    let exercise_response = alice_client
        .exercise(
            "DA.PingPong:Ping",
            &create_response.contract_id,
            "FromUserData",
            json!({"new_count": "3", "new_data": {"name": "Alice", "new_value": 8 }}),
        )
        .await?;
    match exercise_response.events.as_slice() {
        [.., DamlJsonEvent::Created(DamlJsonCreatedEvent {
            payload,
            ..
        })]
        | [DamlJsonEvent::Created(DamlJsonCreatedEvent {
            payload,
            ..
        }), ..] => assert_eq!(*payload, json!({ "sender": "Alice", "receiver": "Bob", "count": "11" })),
        _ => panic!(),
    }
    Ok(())
}

#[tokio::test]
async fn test_create_and_exercise() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let create_and_exercise_response = alice_client
        .create_and_exercise(
            "DA.PingPong:Ping",
            json!({ "sender": "Alice", "receiver": "Bob", "count": 0 }),
            "ResetPingCount",
            json!({}),
        )
        .await?;
    match create_and_exercise_response.events.as_slice() {
        [.., DamlJsonEvent::Created(DamlJsonCreatedEvent {
            payload,
            ..
        })]
        | [DamlJsonEvent::Created(DamlJsonCreatedEvent {
            payload,
            ..
        }), ..] => assert_eq!(*payload, json!({ "sender": "Alice", "receiver": "Bob", "count": "0" })),
        _ => panic!(),
    }
    Ok(())
}

#[tokio::test]
async fn test_exercise_by_key() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let _ =
        alice_client.create("DA.PingPong:Ping", json!({ "sender": "Alice", "receiver": "Bob", "count": 5 })).await?;
    let exercise_by_key_result = alice_client
        .exercise_by_key(
            "DA.PingPong:Ping",
            json!({"sender": "Alice", "count": 5}),
            "FromUserData",
            json!({"new_count": "3", "new_data": {"name": "Alice", "new_value": 8 }}),
        )
        .await?;
    match exercise_by_key_result.events.as_slice() {
        [.., DamlJsonEvent::Created(DamlJsonCreatedEvent {
            payload,
            ..
        })]
        | [DamlJsonEvent::Created(DamlJsonCreatedEvent {
            payload,
            ..
        }), ..] => assert_eq!(*payload, json!({ "sender": "Alice", "receiver": "Bob", "count": "11" })),
        _ => panic!(),
    }
    Ok(())
}

#[tokio::test]
async fn test_fetch() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let create_response =
        alice_client.create("DA.PingPong:Ping", json!({ "sender": "Alice", "receiver": "Bob", "count": 0 })).await?;
    let fetch_response = alice_client.fetch(&create_response.contract_id).await?;
    assert_eq!(fetch_response.payload, json!({ "sender": "Alice", "receiver": "Bob", "count": "0" }));
    Ok(())
}

#[tokio::test]
async fn test_fetch_by_key() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let _ =
        alice_client.create("DA.PingPong:Ping", json!({ "sender": "Alice", "receiver": "Bob", "count": 99 })).await?;
    let fetch_by_key_result =
        alice_client.fetch_by_key("DA.PingPong:Ping", json!({"sender": "Alice", "count": 99})).await?;
    assert_eq!(fetch_by_key_result.payload, json!({ "sender": "Alice", "receiver": "Bob", "count": "99" }));
    Ok(())
}

#[tokio::test]
async fn test_query_all() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let _ =
        alice_client.create("DA.PingPong:Ping", json!({ "sender": "Alice", "receiver": "Bob", "count": 0 })).await?;
    let active_contracts = alice_client.query_all().await?;
    assert_eq!(
        active_contracts.first().unwrap().payload,
        json!({ "sender": "Alice", "receiver": "Bob", "count": "0" })
    );
    Ok(())
}

#[tokio::test]
async fn test_query() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let _ =
        alice_client.create("DA.PingPong:Ping", json!({ "sender": "Alice", "receiver": "Bob", "count": 0 })).await?;
    let active_contracts = alice_client.query(vec!["DA.PingPong:Ping"], json!({})).await?;
    assert_eq!(
        active_contracts.first().unwrap().payload,
        json!({ "sender": "Alice", "receiver": "Bob", "count": "0" })
    );
    Ok(())
}

#[tokio::test]
async fn test_fetch_parties() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let alice_party = alice_client.allocate_party(Some("Alice"), Some("Alice")).await?;
    let fetch_parties_response = alice_client.fetch_parties(vec!["Alice"]).await?;
    assert_eq!(fetch_parties_response, vec![alice_party]);
    Ok(())
}

#[tokio::test]
async fn test_fetch_unknown_party() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let fetch_parties_response = alice_client.fetch_parties(vec!["Paul"]).await?;
    assert_eq!(fetch_parties_response, vec![]);
    Ok(())
}

#[tokio::test]
async fn test_fetch_known_and_unknown_party() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let alice_party = alice_client.allocate_party(Some("Alice"), Some("Alice")).await?;
    let (known_parties, unknown_parties) = alice_client.fetch_parties_with_unknown(vec!["Alice", "Paul"]).await?;
    assert_eq!(known_parties, vec![alice_party]);
    assert_eq!(unknown_parties, vec!["Paul".to_owned()]);
    Ok(())
}

#[tokio::test]
async fn test_fetch_all_parties() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let alice_party = alice_client.allocate_party(Some("Alice"), Some("Alice")).await?;
    let bob_party = alice_client.allocate_party(Some("Bob"), Some("Bob")).await?;
    let fetch_all_parties_response = alice_client.fetch_all_parties().await?;
    assert!(fetch_all_parties_response.contains(&alice_party));
    assert!(fetch_all_parties_response.contains(&bob_party));
    Ok(())
}

#[tokio::test]
async fn test_allocate_party() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let allocate_parties_response = alice_client.allocate_party(Some("Joe"), Some("Joe Smith")).await?;
    assert_eq!(allocate_parties_response, DamlJsonParty::new("Joe", Some("Joe Smith"), true));
    Ok(())
}

#[tokio::test]
async fn test_allocate_party_no_hint() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let allocate_parties_response = alice_client.allocate_party(None, Some("Joe Smith")).await?;
    assert_eq!(allocate_parties_response.display_name, Some("Joe Smith".to_owned()));
    assert_eq!(allocate_parties_response.is_local, true);
    Ok(())
}

#[tokio::test]
async fn test_allocate_party_no_hint_no_display() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let allocate_parties_response = alice_client.allocate_party(None, None).await?;
    assert_eq!(allocate_parties_response.display_name, None);
    assert_eq!(allocate_parties_response.is_local, true);
    Ok(())
}

#[tokio::test]
async fn test_list_packages() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let list_packages_response = alice_client.list_packages().await?;
    assert!(!list_packages_response.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_download_package() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let package_id = alice_client.list_packages().await?.first().unwrap().to_owned();
    let download_package_response = alice_client.download_package(&package_id).await?;
    assert!(!download_package_response.is_empty());
    Ok(())
}

/// Disabled as we cannot reset the `json_api` provided by the SDK between invocations.  Will be re-enabled when we
/// move to using the `json_bridge`.
#[ignore]
#[tokio::test]
async fn test_upload_dar() -> anyhow::Result<()> {
    let _lock = SANDBOX_LOCK.lock().await;
    let alice_client = new_sandbox().await?;
    let dar_file_path = "../resources/testing_types_sandbox/archive/TestingTypes-1_0_0-sdk_1_2_0-lf_1_8.dar";
    let main_package_id = DarFile::from_file(dar_file_path)?.main.hash;
    let mut dar_file = std::fs::File::open(dar_file_path)?;
    let mut buffer = Vec::new();
    dar_file.read_to_end(&mut buffer)?;
    alice_client.upload_dar(buffer).await?;
    let all_packages = alice_client.list_packages().await?;
    assert!(all_packages.contains(&main_package_id));
    Ok(())
}

lazy_static! {
    pub static ref SANDBOX_LOCK: Mutex<()> = Mutex::new(());
}

pub async fn new_sandbox() -> anyhow::Result<DamlJsonClient> {
    reset_sandbox(SANDBOX_GRPC_URL).await?;
    Ok(DamlJsonClientBuilder::url(SANDBOX_REST_URL)
        .connect_timeout(Duration::from_millis(CONNECT_TIMEOUT_MS))
        .timeout(Duration::from_millis(CONNECT_TIMEOUT_MS))
        .with_auth(create_ec256_token(ALICE_PARTY)?)
        .build()?)
}

async fn reset_sandbox(uri: &str) -> anyhow::Result<()> {
    let client = DamlGrpcClientBuilder::uri(uri)
        .timeout(Duration::from_millis(CONNECT_TIMEOUT_MS))
        .with_auth(create_ec256_token(ALICE_PARTY)?)
        .connect()
        .await?;
    client.reset_and_wait().await?;
    Ok(())
}

fn create_ec256_token(party: &str) -> anyhow::Result<String> {
    Ok(DamlSandboxTokenBuilder::new_with_duration_secs(30)
        .ledger_id("wallclock-unsecured-sandbox")
        .application_id("demo")
        .act_as(vec![String::from(party)])
        .read_as(vec![String::from(party)])
        .new_ec256_token(std::fs::read_to_string(TOKEN_KEY_PATH)?)?)
}