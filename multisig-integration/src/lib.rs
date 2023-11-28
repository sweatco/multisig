use anyhow::Result;
use async_trait::async_trait;
use integration_utils::integration_contract::IntegrationContract;
use multisig_model::{
    api::{MultisigApiIntegration, MultisigViewIntegration},
    data::{MultiSigRequest, MultisigRequestId},
};
use near_sdk::{serde_json::json, PublicKey};
use near_workspaces::{Account, Contract};

pub const MULTISIG: &str = "multisig";

pub struct Multisig<'a> {
    account: Option<Account>,
    contract: &'a Contract,
}

#[async_trait]
impl MultisigApiIntegration for Multisig<'_> {
    async fn new(&self, num_confirmations: usize) -> Result<()> {
        self.call("new", json!({ "num_confirmations": num_confirmations})).await
    }

    async fn add_request(&mut self, request: MultiSigRequest) -> Result<MultisigRequestId> {
        self.call("add_request", json!({ "request": request})).await
    }

    async fn add_request_and_confirm(&mut self, request: MultiSigRequest) -> Result<MultisigRequestId> {
        self.call("add_request_and_confirm", json!({ "request": request})).await
    }

    async fn delete_request(&mut self, request_id: MultisigRequestId) -> Result<MultiSigRequest> {
        self.call("delete_request", json!({ "request_id": request_id})).await
    }

    async fn confirm(&mut self, request_id: MultisigRequestId) -> Result<bool> {
        self.call("confirm", json!({ "request_id": request_id })).await
    }
}

#[async_trait]
impl MultisigViewIntegration for Multisig<'_> {
    async fn get_request(&self, request_id: MultisigRequestId) -> Result<MultiSigRequest> {
        self.call("get_request", json!({ "request_id": request_id})).await
    }

    async fn get_num_requests_pk(&self, public_key: PublicKey) -> Result<u32> {
        self.call("get_num_requests_pk", json!({ "public_key": public_key}))
            .await
    }

    async fn list_request_ids(&self) -> Result<Vec<MultisigRequestId>> {
        self.call("list_request_ids", ()).await
    }

    async fn get_confirmations(&self, request_id: MultisigRequestId) -> Result<Vec<PublicKey>> {
        self.call("get_confirmations", json!({ "request_id": request_id})).await
    }

    async fn get_num_confirmations(&self) -> Result<usize> {
        self.call("get_num_confirmations", ()).await
    }

    async fn get_request_nonce(&self) -> Result<u32> {
        self.call("get_request_nonce", ()).await
    }
}

impl<'a> IntegrationContract<'a> for Multisig<'a> {
    fn with_contract(contract: &'a Contract) -> Self {
        Self {
            contract,
            account: None,
        }
    }

    fn with_user(&mut self, account: &Account) -> &mut Self {
        self.account = account.clone().into();
        self
    }

    fn user_account(&self) -> Option<Account> {
        self.account.clone()
    }

    fn contract(&self) -> &'a Contract {
        self.contract
    }
}
