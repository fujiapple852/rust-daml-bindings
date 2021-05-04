use crate::common::test_utils::TestResult;
use crate::common::test_utils::{
    new_static_sandbox, update_create_command_package_id_for_testing, update_exercise_command_package_id_for_testing,
    SANDBOX_LOCK,
};
use daml::grpc_api::data::command::DamlCommand;
use daml::grpc_api::data::event::DamlEvent;
use daml::grpc_api::{CommandExecutor, DamlSimpleExecutorBuilder};
use daml_derive::daml_codegen;
use std::convert::TryInto;

daml_codegen!(dar_file = r"resources/testing_types_sandbox/archive/TestingTypes-1_0_0-sdk_1_13_0-lf_1_12.dar");

#[tokio::test]
async fn test_rent() -> TestResult {
    let _lock = SANDBOX_LOCK.lock().await;
    let client = new_static_sandbox().await?;
    let alice_executor = DamlSimpleExecutorBuilder::new(&client).act_as("Alice").build()?;
    let bob_executor = DamlSimpleExecutorBuilder::new(&client).act_as("Bob").build()?;
    let proposal = testing_types::da::rent_demo::RentalProposal::new("Alice", "Bob", "Some Terms");
    let create_proposal_command = proposal.create_command();
    let create_proposal_command =
        update_create_command_package_id_for_testing(&client, create_proposal_command).await?;
    let proposal_result = alice_executor.execute_for_transaction(DamlCommand::Create(create_proposal_command)).await?;
    let event: DamlEvent = proposal_result.take_events().swap_remove(0);
    let proposal_contract: testing_types::da::rent_demo::RentalProposalContract = event.try_created()?.try_into()?;
    assert_eq!(&proposal, proposal_contract.data());
    let exercise_command = proposal_contract.id().accept_command("dummy text", 0);
    let exercise_command = update_exercise_command_package_id_for_testing(&client, exercise_command).await?;
    let accept_result = bob_executor.execute_for_transaction(DamlCommand::Exercise(exercise_command)).await?;
    let accept_event: DamlEvent = accept_result.take_events().swap_remove(1);
    let agreement_contract: testing_types::da::rent_demo::RentalAgreementContract =
        accept_event.try_created()?.try_into()?;
    assert_eq!("Alice", agreement_contract.data().landlord.as_str());
    assert_eq!("Bob", agreement_contract.data().tenant.as_str());
    assert_eq!("Some Terms", &agreement_contract.data().terms);
    Ok(())
}
