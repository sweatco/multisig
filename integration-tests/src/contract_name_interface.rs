#![cfg(test)]

use anyhow::Result;
use async_trait::async_trait;
use integration_utils::integration_contract::IntegrationContract;
use multisig_model::{
    api::{MultisigApiIntegration, MultisigViewIntegration},
    data::{MultiSigRequest, MultisigRequestId},
};
use near_sdk::PublicKey;
use near_workspaces::{Account, Contract};
use serde_json::json;

pub const MULTISIG: &str = "multisig";

pub struct Multisig<'a> {
    account: Option<Account>,
    contract: &'a Contract,
}

#[async_trait]
impl MultisigApiIntegration for Multisig<'_> {
    async fn new(&self, num_confirmations: usize) -> Result<()> {
        self.call_contract("new", json!({ "num_confirmations": num_confirmations}))
            .await
    }

    async fn add_request(&mut self, request: MultiSigRequest) -> Result<MultisigRequestId> {
        self.call_contract("add_request", json!({ "request": request})).await
    }

    async fn add_request_and_confirm(&mut self, request: MultiSigRequest) -> Result<MultisigRequestId> {
        self.call_contract("add_request_and_confirm", json!({ "request": request}))
            .await
    }

    async fn delete_request(&mut self, request_id: MultisigRequestId) -> Result<MultiSigRequest> {
        self.call_contract("delete_request", json!({ "request_id": request_id}))
            .await
    }

    async fn confirm(&mut self, request_id: MultisigRequestId) -> Result<bool> {
        self.call_contract("confirm", json!({ "request_id": request_id })).await
    }
}

#[async_trait]
impl MultisigViewIntegration for Multisig<'_> {
    async fn get_request(&self, request_id: MultisigRequestId) -> Result<MultiSigRequest> {
        self.call_contract("get_request", json!({ "request_id": request_id}))
            .await
    }

    async fn get_num_requests_pk(&self, public_key: PublicKey) -> Result<u32> {
        self.call_contract("get_num_requests_pk", json!({ "public_key": public_key}))
            .await
    }

    async fn list_request_ids(&self) -> Result<Vec<MultisigRequestId>> {
        self.call_contract("list_request_ids", ()).await
    }

    async fn get_confirmations(&self, request_id: MultisigRequestId) -> Result<Vec<PublicKey>> {
        self.call_contract("get_confirmations", json!({ "request_id": request_id}))
            .await
    }

    async fn get_num_confirmations(&self) -> Result<usize> {
        self.call_contract("get_num_confirmations", ()).await
    }

    async fn get_request_nonce(&self) -> Result<u32> {
        self.call_contract("get_request_nonce", ()).await
    }
}

impl<'a> IntegrationContract<'a> for Multisig<'a> {
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
