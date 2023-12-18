#![cfg(test)]

use multisig_model::api::MultisigViewIntegration;

use crate::context::{prepare_contract, IntegrationContext};

#[tokio::test]
async fn happy_flow() -> anyhow::Result<()> {
    println!("👷🏽 Run happy flow test");

    let mut context = prepare_contract().await?;

    let alice = context.alice().await?;

    assert_eq!(
        0,
        context.multisig().get_request_nonce().with_user(&alice).call().await?
    );
    assert_eq!(2, context.multisig().get_num_confirmations().call().await?);

    Ok(())
}
