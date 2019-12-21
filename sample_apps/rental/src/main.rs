use daml::prelude::*;
use daml_ledger_api::{DamlLedgerClient, DamlSimpleExecutorBuilder};

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
