#![warn(clippy::all, clippy::pedantic)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

use std::sync::Arc;

use chrono::{DateTime, Utc};
use futures::future;
use futures::future::Future;
use futures::stream::Stream;
use time::Duration;

use daml_ledger_api::data::command::{DamlCommand, DamlCreateCommand, DamlExerciseCommand};
use daml_ledger_api::data::event::{DamlCreatedEvent, DamlEvent};
use daml_ledger_api::data::filter::DamlTransactionFilter;
use daml_ledger_api::data::offset::{DamlLedgerOffset, DamlLedgerOffsetBoundary, DamlLedgerOffsetType};
use daml_ledger_api::data::value::{DamlRecord, DamlValue};
use daml_ledger_api::data::DamlError;
use daml_ledger_api::data::DamlIdentifier;
use daml_ledger_api::data::DamlResult;
use daml_ledger_api::DamlCommandFactory;
use daml_ledger_api::DamlLedgerClient;

use daml_ledger_api::data::DamlTransaction;
use daml_ledger_api::service::DamlVerbosity;
use daml_ledger_macro::{daml_path, daml_value};
use daml_lf::DamlLfArchivePayload;
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
const TIMEOUT_SECS: i64 = 30;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("sample_apps/ping_pong/log4rs.yml", log4rs::file::Deserializers::default())?;

    // Create a connection to a Sandbox (in wall clock mode) and reset the ledger.  This connection will be used by
    // both parties and so we wrap in an Arc<> here.
    let ledger_client = create_connection("localhost", 8080)?;

    // Get the PingPong package id from the ledger.
    let package_id = get_ping_pong_package_id(&ledger_client)?;

    // Alice creates an initial Ping contract with Bob as receiver an a count of 0.
    send_initial_ping(&ledger_client, &package_id, PARTY_ALICE)?;

    // Create a stream of transactions processing pipeline for both Alice and Bob.
    let bob_processor = process_ping_pong(ledger_client.clone(), package_id.clone(), PARTY_BOB.to_owned())?;
    let alice_processor = process_ping_pong(ledger_client, package_id, PARTY_ALICE.to_owned())?;

    // Create a stream which selects over both Alice and Bob's processor streams and drive it to completion on the
    // main application thread.  Note that this is the only thread used in this application but the stream operate
    // asynchronously.
    bob_processor.select(alice_processor).collect().wait()?;

    Ok(())
}

fn create_connection(hostname: &str, port: u16) -> DamlResult<Arc<DamlLedgerClient>> {
    let mut ledger_client = DamlLedgerClient::connect(hostname, port)?;
    ledger_client = ledger_client.reset_and_wait()?;
    Ok(Arc::new(ledger_client))
}

fn process_ping_pong(
    ledger_client: Arc<DamlLedgerClient>,
    package_id: String,
    party: String,
) -> DamlResult<impl Stream<Item = (), Error = DamlError>> {
    let transactions_future = ledger_client
        .transaction_service()
        // Create an infinite stream of transactions for the given party from the beginning of the ledger.  We request
        // verbose transactions so that we can access the field labels.
        .get_transactions(
            DamlLedgerOffset::Boundary(DamlLedgerOffsetBoundary::Begin),
            DamlLedgerOffsetType::Unbounded,
            DamlTransactionFilter::for_parties(vec![party.clone()]),
            DamlVerbosity::Verbose,
        )?
        // Each stream item contains a vector of transactions which in-turn contains a vector of events and so we
        // flatten these out and process only creation events.  For this example there will only be a single
        // transaction per stream item and within that only a single created event (there will be archived events as
        // well).  Note that get_transactions() does not return exercised events (use get_transaction_trees for that).
        //
        // collects into a Option<Vec<()>> which will be Some(_) if the stream should could continue or None if the
        // stream should end (ie. count > 10).
        .map(move |transactions| {
            let events: Vec<&DamlEvent> = transactions.iter().flat_map(DamlTransaction::events).collect();
            events
                .iter()
                .filter_map(|&event| match event {
                    DamlEvent::Created(e) =>
                        Some(process_event(&ledger_client, &package_id, &party, e).expect("failed to process event")),
                    _ => None,
                })
                .collect::<Option<Vec<()>>>()
        })
        // Keep drawing from the stream whilst until the stream procedures a None value, which is our signal to stop
        .take_while(|f| future::ok(f.is_some()))
        // We don't ultimately return anything from the stream and so we remap the output to () here.
        .map(|_| ());
    Ok(transactions_future)
}

fn process_event(
    ledger_client: &DamlLedgerClient,
    package_id: &str,
    party: &str,
    created_event: &DamlCreatedEvent,
) -> std::result::Result<Option<()>, Box<dyn std::error::Error>> {
    let entity_name = created_event.template_id().entity_name();
    let contract_id = created_event.contract_id();
    let receiver = created_event.create_arguments().extract(daml_path![receiver#p])?;
    let count = *created_event.create_arguments().extract(daml_path![count#i])?;
    if count <= 10 {
        if party == receiver {
            info!("{} received:\t{} ({}) with count {}", receiver, entity_name, contract_id, count);
            exercise_choice(&ledger_client, &package_id, &entity_name, &party, contract_id, response(entity_name))?;
        }
        Ok(Some(()))
    } else {
        Ok(None)
    }
}

fn create_command_factory(workflow_id: &str, application_id: &str, sending_party: &str) -> DamlCommandFactory {
    let ledger_effective_time: DateTime<Utc> = Utc::now();
    let maximum_record_time = ledger_effective_time.add(Duration::seconds(TIMEOUT_SECS));
    DamlCommandFactory::new(workflow_id, application_id, sending_party, ledger_effective_time, maximum_record_time)
}

fn send_initial_ping(ledger_client: &DamlLedgerClient, package_id: &str, party: &str) -> DamlResult<()> {
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
    // Note that this performs synchronous `submit_and_wait` for simplicity, we could instead return a future here.
    ledger_client.command_service().submit_and_wait_sync(create_commands)?;
    Ok(())
}

fn exercise_choice(
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
    ledger_client.command_service().submit_and_wait_sync(exercise_commands)
}

fn response(entity_name: &str) -> &str {
    match entity_name {
        PING_ENTITY_NAME => CHOICE_RESPOND_PONG,
        PONG_ENTITY_NAME => CHOICE_RESPOND_PING,
        _ => unreachable!(),
    }
}

fn get_ping_pong_package_id(
    ledger_client: &DamlLedgerClient,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    fn get_package_payload<'a>(
        ledger_client: &DamlLedgerClient,
        package_id: &'a str,
    ) -> std::result::Result<(&'a str, DamlLfArchivePayload), Box<dyn std::error::Error>> {
        let package = ledger_client.package_service().get_package_sync(package_id)?;
        let archive = DamlLfArchivePayload::from_bytes(package.payload())?;
        Ok((package_id, archive))
    }

    let all_packages = ledger_client.package_service().list_packages_sync()?;
    let all_archives: Vec<(&str, DamlLfArchivePayload)> = all_packages
        .iter()
        .map(|package_id| (get_package_payload(ledger_client, package_id)))
        .collect::<std::result::Result<Vec<(&str, DamlLfArchivePayload)>, _>>()?;
    all_archives
        .iter()
        .find(|(_, archive)| archive.contains_module(PINGPONG_MODULE_NAME))
        .map_or(Err("package could not be found".into()), |(package_id, _)| Ok((*package_id).to_string()))
}
