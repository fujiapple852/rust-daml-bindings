use daml::prelude::*;
use daml_ledger_api::{DamlLedgerClient, DamlSimpleExecutorBuilder};

// pub mod autogen {
//    pub mod rental_0_0_1;
//}
// use autogen::rental_0_0_1::da::rental::*;

include!("autogen/rental_0_0_1.rs");
use rental_0_0_1::da::rental::*;

fn main() -> DamlResult<()> {
    let client = DamlLedgerClient::connect("localhost", 8082)?.reset_and_wait()?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();
    let bob_executor = DamlSimpleExecutorBuilder::new(&client, "Bob").build();
    let proposal_data = RentalProposal::new("Alice", "Bob", "test");
    let proposal_contract = alice_executor.execute(proposal_data.create())?;
    dbg!(&proposal_contract);
    let agreement_contract_id =
        RentalProposalContractId::try_from(bob_executor.execute(proposal_contract.id().accept("", 0))?)?;
    dbg!(&agreement_contract_id);
    Ok(())
}
