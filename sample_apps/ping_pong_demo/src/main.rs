#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

use chrono::Duration;
use chrono::{DateTime, Utc};

use daml_ledger_api::data::command::{DamlCommand, DamlCreateCommand, DamlExerciseCommand};
use daml_ledger_api::data::event::{DamlCreatedEvent, DamlEvent};
use daml_ledger_api::data::filter::DamlTransactionFilter;
use daml_ledger_api::data::offset::{DamlLedgerOffset, DamlLedgerOffsetBoundary, DamlLedgerOffsetType};
use daml_ledger_api::data::value::{DamlRecord, DamlValue};
use daml_ledger_api::data::DamlIdentifier;
use daml_ledger_api::data::DamlResult;
use daml_ledger_api::{DamlCommandFactory, DamlLedgerClient, DamlLedgerClientBuilder, DamlSandboxTokenBuilder};

use daml_ledger_api::data::DamlTransaction;
use daml_ledger_api::service::DamlVerbosity;
use daml_ledger_macro::{daml_path, daml_value};
use daml_ledger_util::package::find_module_package_id;
use futures::stream::StreamExt;
use futures::try_join;
use log::info;
use std::convert::TryInto;
use std::ops::Add;

const PINGPONG_MODULE_NAME: &str = "DA.PingPong";
const PING_ENTITY_NAME: &str = "Ping";
const PONG_ENTITY_NAME: &str = "Pong";
const PINGPONG_WORKFLOW_ID: &str = "PingPongWorkflow";
const PINGPONG_APP_ID: &str = "PingPongApp";
const PARTY_ALICE: &str = "Alice";
const PARTY_BOB: &str = "Bob";
const CHOICE_RESPOND_PING: &str = "RespondPing";
const CHOICE_RESPOND_PONG: &str = "RespondPong";
const TRANSACTION_WINDOW_SECS: i64 = 30;
const TOKEN_VALIDITY_SECS: i64 = 60;
const TOKEN_KEY_PATH: &str = "resources/testing_types_sandbox/certs/ec256.key";

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("sample_apps/ping_pong_demo/log4rs.yml", log4rs::file::Deserializers::default())?;
    let ledger_client = create_connection("http://localhost:8080").await?;
    let package_id = find_module_package_id(&ledger_client, PINGPONG_MODULE_NAME).await?;
    send_initial_ping(&ledger_client, &package_id, PARTY_ALICE).await?;
    let bob_fut = process_ping_pong(&ledger_client, package_id.clone(), PARTY_BOB.to_owned());
    let alice_fut = process_ping_pong(&ledger_client, package_id, PARTY_ALICE.to_owned());
    try_join!(bob_fut, alice_fut)?;
    Ok(())
}

async fn create_connection(uri: &str) -> DamlResult<DamlLedgerClient> {
    let ledger_client =
        DamlLedgerClientBuilder::uri(uri).with_auth(create_ec256_token()?).connect().await?.reset_and_wait().await?;
    Ok(ledger_client)
}

async fn send_initial_ping(ledger_client: &DamlLedgerClient, package_id: &str, party: &str) -> DamlResult<()> {
    let ping_record: DamlRecord = daml_value![{
        sender: PARTY_ALICE#p,
        receiver: PARTY_BOB#p,
        count: 0
    }]
    .try_into()?;
    let template_id = DamlIdentifier::new(package_id, PINGPONG_MODULE_NAME, PING_ENTITY_NAME);
    let create_command = DamlCommand::Create(DamlCreateCommand::new(template_id, ping_record));
    let commands_factory = create_command_factory(PINGPONG_WORKFLOW_ID, PINGPONG_APP_ID, party);
    let create_commands = commands_factory.make_command(create_command);
    ledger_client.command_service().submit_and_wait(create_commands).await?;
    Ok(())
}

async fn process_ping_pong(ledger_client: &DamlLedgerClient, package_id: String, party: String) -> DamlResult<()> {
    let mut transactions_stream = ledger_client
        .transaction_service()
        .get_transactions(
            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin),
            DamlLedgerOffsetType::Unbounded,
            DamlTransactionFilter::for_parties(vec![party.clone()]),
            DamlVerbosity::Verbose,
        )
        .await?;
    while let Some(item) = transactions_stream.next().await.transpose()? {
        let events: Vec<&DamlEvent> = item.iter().flat_map(DamlTransaction::events).collect();
        for event in events {
            if let DamlEvent::Created(e) = event {
                if process_event(ledger_client, &package_id, &party, e).await?.is_none() {
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

async fn process_event(
    ledger_client: &DamlLedgerClient,
    package_id: &str,
    party: &str,
    created_event: &DamlCreatedEvent,
) -> DamlResult<Option<()>> {
    let entity_name = created_event.template_id().entity_name();
    let contract_id = created_event.contract_id();
    let receiver = created_event.create_arguments().extract(daml_path![receiver#p])?;
    let count = *created_event.create_arguments().extract(daml_path![count#i])?;
    if count <= 10 {
        if party == receiver {
            info!("{} received:\t{} ({}) with count {}", receiver, entity_name, contract_id, count);
            exercise_choice(ledger_client, package_id, entity_name, party, contract_id, response(entity_name)).await?;
        }
        Ok(Some(()))
    } else {
        Ok(None)
    }
}

async fn exercise_choice(
    ledger_client: &DamlLedgerClient,
    package_id: &str,
    entity_name: &str,
    party: &str,
    contract_id: &str,
    choice: &str,
) -> DamlResult<String> {
    let commands_factory = create_command_factory(PINGPONG_WORKFLOW_ID, PINGPONG_APP_ID, party);
    let exercise_command = DamlCommand::Exercise(DamlExerciseCommand::new(
        DamlIdentifier::new(package_id, PINGPONG_MODULE_NAME, entity_name),
        contract_id,
        choice,
        DamlValue::new_record(DamlRecord::empty()),
    ));
    let exercise_commands = commands_factory.make_command(exercise_command);
    ledger_client.command_service().submit_and_wait(exercise_commands).await
}

fn create_command_factory(workflow_id: &str, application_id: &str, sending_party: &str) -> DamlCommandFactory {
    let ledger_effective_time: DateTime<Utc> = Utc::now();
    let maximum_record_time = ledger_effective_time.add(Duration::seconds(TRANSACTION_WINDOW_SECS));
    DamlCommandFactory::new(workflow_id, application_id, sending_party, ledger_effective_time, maximum_record_time)
}

fn response(entity_name: &str) -> &str {
    match entity_name {
        PING_ENTITY_NAME => CHOICE_RESPOND_PONG,
        PONG_ENTITY_NAME => CHOICE_RESPOND_PING,
        _ => unreachable!(),
    }
}

fn create_ec256_token() -> DamlResult<String> {
    DamlSandboxTokenBuilder::new_with_duration_secs(TOKEN_VALIDITY_SECS)
        .admin(true)
        .act_as(vec![String::from(PARTY_ALICE), String::from(PARTY_BOB)])
        .read_as(vec![String::from(PARTY_ALICE), String::from(PARTY_BOB)])
        .new_ec256_token(std::fs::read_to_string(TOKEN_KEY_PATH)?)
}
