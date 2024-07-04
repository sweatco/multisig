#![cfg(test)]

use anyhow::Result;
use multisig_model::{MultisigApiIntegration, MultisigContract};
use near_workspaces::Account;

pub type Context = nitka::context::Context<near_workspaces::network::Sandbox>;

pub const MULTISIG: &str = "multisig";

pub(crate) trait IntegrationContext {
    async fn alice(&mut self) -> Result<Account>;
    fn multisig(&self) -> MultisigContract<'_>;
}

impl IntegrationContext for Context {
    async fn alice(&mut self) -> Result<Account> {
        self.account("alice").await
    }

    fn multisig(&self) -> MultisigContract<'_> {
        MultisigContract {
            contract: &self.contracts[MULTISIG],
        }
    }
}

pub(crate) async fn prepare_contract() -> Result<Context> {
    let context = Context::new(&[MULTISIG], true, "build-integration".into()).await?;
    context.multisig().new(2).await?;
    Ok(context)
}
