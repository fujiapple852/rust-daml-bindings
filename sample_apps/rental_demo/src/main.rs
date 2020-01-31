use daml::prelude::*;

include!("autogen/rental_0_0_1.rs");

use daml_ledger_api::{DamlLedgerClientBuilder, DamlSimpleExecutorBuilder};
use rental_0_0_1::da::rental::*;

#[tokio::main]
async fn main() -> DamlResult<()> {
    let client = DamlLedgerClientBuilder::uri("http://localhost:8082").connect().await?.reset_and_wait().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client, "Alice").build();
    let bob_executor = DamlSimpleExecutorBuilder::new(&client, "Bob").build();
    let proposal_data = RentalProposal::new("Alice", "Bob", "test");
    let proposal_event = alice_executor.execute_create(proposal_data.create_command()).await?;
    let proposal_contract = RentalProposalContract::try_from(proposal_event)?;
    let accept_result = bob_executor.execute_exercise(proposal_contract.id().accept_command("", 0)).await?;
    let agreement_contract_id = RentalAgreementContractId::try_from(accept_result.try_contract_id()?.to_owned())?;
    println!("{:?}", &agreement_contract_id);
    Ok(())

    //    // Previous API (pre-async)
    //    let proposal_contract = alice_executor.execute(proposal_data.create()).await?;
    //    let agreement_contract_id =
    // RentalAgreementContractId::try_from(bob_executor.execute(proposal_contract.id().accept("", 0))?)?;
}

// use daml_ledger_codegen::generator::{daml_codegen, ModuleOutputMode, RenderMethod};
//
// pub mod autogen {
//    pub mod daml_prim;
//    pub mod daml_prim_ghc_prim;
//    pub mod daml_stdlib_0_13_41;
//    pub mod daml_stdlib_0_13_41_da_internal_any;
//    pub mod daml_stdlib_0_13_41_da_internal_template;
//    pub mod testing_types_1_0_0;
//    pub mod rental_0_0_1;
//}
// const DAR_PATH: &str = "resources/testing_types_sandbox/archive/TestingTypes-1_0_0-sdk_0_13_41-lf_1_7.dar";
// const OUTPUT_PATH: &str = "sample_apps/rental/src/autogen";
//
// fn main() {
//    daml_codegen(DAR_PATH, OUTPUT_PATH, &[], RenderMethod::Full, ModuleOutputMode::Separate)
//        .expect("failed to generate code for DAML archive");
//}
