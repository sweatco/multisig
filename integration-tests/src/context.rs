#![cfg(test)]

use async_trait::async_trait;
use integration_utils::integration_contract::IntegrationContract;
use multisig_integration::{Multisig, MULTISIG};
use multisig_model::api::MultisigApiIntegration;
use near_workspaces::Account;

pub type Context = integration_utils::context::Context<near_workspaces::network::Sandbox>;

#[async_trait]
pub trait IntegrationContext {
    async fn manager(&mut self) -> anyhow::Result<Account>;
    async fn alice(&mut self) -> anyhow::Result<Account>;
    async fn fee(&mut self) -> anyhow::Result<Account>;
    fn multisig(&self) -> Multisig<'_>;
}

#[async_trait]
impl IntegrationContext for Context {
    async fn manager(&mut self) -> anyhow::Result<Account> {
        self.account("manager").await
    }

    async fn alice(&mut self) -> anyhow::Result<Account> {
        self.account("alice").await
    }

    async fn fee(&mut self) -> anyhow::Result<Account> {
        self.account("fee").await
    }

    fn multisig(&self) -> Multisig<'_> {
        Multisig::with_contract(&self.contracts[MULTISIG])
    }
}

pub(crate) async fn prepare_contract() -> anyhow::Result<Context> {
    let context = Context::new(&[MULTISIG], "build-integration".into()).await?;
    context.multisig().new(2).await?;
    Ok(context)
}
