include!("autogen/rental_0_0_1.rs");

use anyhow::{Context, Result};
use daml::grpc_api::{CommandExecutor, DamlGrpcClientBuilder, DamlSimpleExecutorBuilder};
use rental::da::rental::*;
use std::convert::TryFrom;

const LOGGER_CONFIG: &str = "resources/log4rs.yml";

#[tokio::main]
async fn main() -> Result<()> {
    log4rs::init_file(LOGGER_CONFIG, Default::default()).context(LOGGER_CONFIG)?;
    let client = DamlGrpcClientBuilder::uri("http://localhost:8082").connect().await?.reset_and_wait().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client).act_as("Alice").build()?;
    let bob_executor = DamlSimpleExecutorBuilder::new(&client).act_as("Bob").build()?;
    let proposal_data = RentalProposal::new("Alice", "Bob", "test");
    let proposal_event = alice_executor.execute_create(proposal_data.create_command()).await?;
    let proposal_contract = RentalProposalContract::try_from(proposal_event)?;
    let accept_result = bob_executor.execute_exercise(proposal_contract.id().accept_command("", 0)).await?;
    let agreement_contract_id = RentalAgreementContractId::try_from(accept_result.try_contract_id()?.to_owned())?;
    println!("{:?}", &agreement_contract_id);
    Ok(())
}
