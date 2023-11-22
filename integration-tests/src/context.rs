#![cfg(test)]

use async_trait::async_trait;
use integration_utils::integration_contract::IntegrationContract;
use model::ContractNameInterfaceIntegration;
use near_workspaces::Account;

use crate::contract_name_interface::{ContractName, CONTRACT_NAME};

pub type Context = integration_utils::context::Context<near_workspaces::network::Sandbox>;

#[async_trait]
pub trait IntegrationContext {
    async fn manager(&mut self) -> anyhow::Result<Account>;
    async fn alice(&mut self) -> anyhow::Result<Account>;
    async fn fee(&mut self) -> anyhow::Result<Account>;
    fn contract_name(&self) -> ContractName<'_>;
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

    fn contract_name(&self) -> ContractName<'_> {
        ContractName::with_contract(&self.contracts[CONTRACT_NAME])
    }
}

pub(crate) async fn prepare_contract() -> anyhow::Result<Context> {
    let context = Context::new(&[CONTRACT_NAME], "build-integration".into()).await?;
    context.contract_name().init().await?;
    Ok(context)
}
