#[tokio::test]
async fn happy_flow() -> anyhow::Result<()> {
    use model::ContractNameInterfaceIntegration;

    use crate::context::{prepare_contract, IntegrationContext};

    println!("👷🏽 Run happy flow test");

    let context = prepare_contract().await?;

    assert_eq!(context.contract_name().receive_name().await?, "Default name");

    context.contract_name().set_name("New name".to_string()).await?;

    assert_eq!(context.contract_name().receive_name().await?, "New name");

    Ok(())
}
