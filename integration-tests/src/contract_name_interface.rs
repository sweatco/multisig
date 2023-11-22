#![cfg(test)]

use async_trait::async_trait;
use integration_utils::integration_contract::IntegrationContract;
use model::ContractNameInterfaceIntegration;
use near_workspaces::{Account, Contract};
use serde_json::json;

pub const CONTRACT_NAME: &str = "contract_name";

pub struct ContractName<'a> {
    account: Option<Account>,
    contract: &'a Contract,
}

#[async_trait]
impl ContractNameInterfaceIntegration for ContractName<'_> {
    async fn init(&self) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        println!("▶️ Init contract");

        self.contract.call("init").max_gas().transact().await?.into_result()?;

        Ok(())
    }

    async fn initialize_with_name(&self, name: String) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        println!("▶️ Init contract with name");

        self.contract
            .call("init_with_name")
            .args_json(json!({
                "name": name,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(())
    }

    async fn receive_name(&self) -> anyhow::Result<String> {
        println!("▶️ Init contract with name");

        let result = self
            .contract
            .call("receive_name")
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(result.json()?)
    }

    async fn set_name(&mut self, name: String) -> anyhow::Result<()> {
        println!("▶️ Init contract with name");

        self.contract
            .call("set_name")
            .args_json(json!({
                "name": name,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(())
    }
}

impl<'a> IntegrationContract<'a> for ContractName<'a> {
    fn with_contract(contract: &'a Contract) -> Self {
        Self {
            contract,
            account: None,
        }
    }

    fn with_user(mut self, account: &Account) -> Self {
        self.account = account.clone().into();
        self
    }

    fn user_account(&self) -> Account {
        self.account
            .as_ref()
            .expect("Set account with `user` method first")
            .clone()
    }

    fn contract(&self) -> &'a Contract {
        self.contract
    }
}
