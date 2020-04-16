use daml::prelude::*;

include!("autogen/rental_0_0_1.rs");

use daml::api::{DamlLedgerClientBuilder, DamlSimpleExecutorBuilder};
use rental::da::rental::*;

#[tokio::main]
async fn main() -> DamlResult<()> {
    log4rs::init_file("sample_apps/rental_demo/resources/log4rs.yml", log4rs::file::Deserializers::default())
        .map_err(|e| DamlError::Other(e.to_string()))?;
    let client = DamlLedgerClientBuilder::uri("http://localhost:8082").connect().await?.reset_and_wait().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();
    let bob_executor = DamlSimpleExecutorBuilder::new(&client, "Bob").build();
    let proposal_data = RentalProposal::new("Alice", "Bob", "test");
    let proposal_event = alice_executor.execute_create(proposal_data.create_command()).await?;
    let proposal_contract = RentalProposalContract::try_from(proposal_event)?;
    let accept_result = bob_executor.execute_exercise(proposal_contract.id().accept_command("", 0)).await?;
    let agreement_contract_id =
        RentalAgreementContractId::try_from(DamlContractId::new(accept_result.try_contract_id()?.to_owned()))?;
    println!("{:?}", &agreement_contract_id);
    Ok(())
}
