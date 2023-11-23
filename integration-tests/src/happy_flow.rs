#![cfg(test)]

use model::api::MultisigViewIntegration;

use crate::context::{prepare_contract, IntegrationContext};

#[tokio::test]
async fn happy_flow() -> anyhow::Result<()> {
    println!("👷🏽 Run happy flow test");

    let context = prepare_contract().await?;

    assert_eq!(0, context.multisig().get_request_nonce().await?);
    assert_eq!(2, context.multisig().get_num_confirmations().await?);

    Ok(())
}
