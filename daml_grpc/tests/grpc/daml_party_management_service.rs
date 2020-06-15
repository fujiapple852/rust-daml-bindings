use crate::common::ping_pong::{new_wallclock_sandbox, TestResult, WALLCLOCK_SANDBOX_LOCK};
use daml_grpc::data::party::DamlPartyDetails;

#[tokio::test]
async fn test_get_participant_id() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock();
    let ledger_client = new_wallclock_sandbox().await?;
    let participant_id = ledger_client.party_management_service().get_participant_id().await?;
    assert_eq!("sandbox-participant", participant_id);
    Ok(())
}

#[tokio::test]
async fn test_list_known_parties() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock();
    let ledger_client = new_wallclock_sandbox().await?;
    let known_parties = ledger_client.party_management_service().list_known_parties().await?;
    assert!(known_parties.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_get_parties() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock();
    let ledger_client = new_wallclock_sandbox().await?;
    let alice_party = ledger_client.party_management_service().allocate_party("Alice", "Alice Smith").await?;
    let bob_party = ledger_client.party_management_service().allocate_party("Bob", "Bob Jones").await?;
    let parties =
        ledger_client.party_management_service().get_parties(vec!["Alice".to_owned(), "Bob".to_owned()]).await?;
    assert!(parties.contains(&alice_party));
    assert!(parties.contains(&bob_party));
    Ok(())
}

#[tokio::test]
async fn test_allocate_party() -> TestResult {
    let _lock = WALLCLOCK_SANDBOX_LOCK.lock();
    let ledger_client = new_wallclock_sandbox().await?;
    let allocated_party = ledger_client.party_management_service().allocate_party("Alice", "Alice Smith").await?;
    assert_eq!(DamlPartyDetails::new("Alice", "Alice Smith", true), allocated_party);
    Ok(())
}
