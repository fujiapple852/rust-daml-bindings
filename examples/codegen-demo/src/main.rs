include!("autogen/rental_0_1_0.rs");

use anyhow::Result;
use daml::grpc_api::{CommandExecutor, DamlGrpcClientBuilder, DamlSimpleExecutorBuilder};
use rental::fuji::rental::*;
use std::convert::TryFrom;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NONE)
        .with_env_filter("daml-grpc::service::daml_command_service=trace")
        .init();
    let client = DamlGrpcClientBuilder::uri("http://localhost:8082").connect().await?.reset_and_wait().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client).act_as("Alice").build()?;
    let bob_executor = DamlSimpleExecutorBuilder::new(&client).act_as("Bob").build()?;
    let proposal_data = RentalProposal::new("Alice", "Bob", "test");
    let proposal_event = alice_executor.execute_create(proposal_data.create_command()).await?;
    let proposal_contract = RentalProposalContract::try_from(proposal_event)?;
    let accept_result = bob_executor.execute_exercise(proposal_contract.id().accept_command("", 0)).await?;
    let agreement_contract_id = RentalAgreementContractId::try_from(accept_result.try_contract_id()?.to_owned())?;
    info!(?agreement_contract_id);
    Ok(())
}
