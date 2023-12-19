use async_trait::async_trait;
use integration_utils::{contract_call::ContractCall, integration_contract::IntegrationContract};
use multisig_model::{
    api::{MultisigApiIntegration, MultisigViewIntegration},
    data::{MultiSigRequest, MultisigRequestId},
};
use near_sdk::{serde_json::json, PublicKey};
use near_workspaces::{types::NearToken, Contract};

pub const MULTISIG: &str = "multisig";

pub struct Multisig<'a> {
    contract: &'a Contract,
}

#[async_trait]
impl MultisigApiIntegration for Multisig<'_> {
    fn new(&self, num_confirmations: usize) -> ContractCall<()> {
        self.make_call("new")
            .args_json(json!({ "num_confirmations": num_confirmations}))
            .unwrap()
    }

    fn add_request(&mut self, request: MultiSigRequest) -> ContractCall<MultisigRequestId> {
        self.make_call("add_request")
            .args_json(json!({ "request": request}))
            .unwrap()
    }

    fn add_request_and_confirm(&mut self, request: MultiSigRequest) -> ContractCall<MultisigRequestId> {
        self.make_call("add_request_and_confirm")
            .args_json(json!({ "request": request}))
            .unwrap()
    }

    fn delete_request(&mut self, request_id: MultisigRequestId) -> ContractCall<MultiSigRequest> {
        self.make_call("delete_request")
            .args_json(json!({ "request_id": request_id}))
            .unwrap()
    }

    fn confirm(&mut self, request_id: MultisigRequestId) -> ContractCall<()> {
        self.make_call("confirm")
            .args_json(json!({ "request_id": request_id }))
            .unwrap()
            .deposit(NearToken::from_yoctonear(1))
    }
}

#[async_trait]
impl MultisigViewIntegration for Multisig<'_> {
    fn get_request(&self, request_id: MultisigRequestId) -> ContractCall<MultiSigRequest> {
        self.make_call("get_request")
            .args_json(json!({ "request_id": request_id}))
            .unwrap()
    }

    fn get_num_requests_pk(&self, public_key: PublicKey) -> ContractCall<u32> {
        self.make_call("get_num_requests_pk")
            .args_json(json!({ "public_key": public_key}))
            .unwrap()
    }

    fn list_request_ids(&self) -> ContractCall<Vec<MultisigRequestId>> {
        self.make_call("list_request_ids")
    }

    fn get_confirmations(&self, request_id: MultisigRequestId) -> ContractCall<Vec<PublicKey>> {
        self.make_call("get_confirmations")
            .args_json(json!({ "request_id": request_id}))
            .unwrap()
    }

    fn get_num_confirmations(&self) -> ContractCall<usize> {
        self.make_call("get_num_confirmations")
    }

    fn get_request_nonce(&self) -> ContractCall<u32> {
        self.make_call("get_request_nonce")
    }
}

impl<'a> IntegrationContract<'a> for Multisig<'a> {
    fn with_contract(contract: &'a Contract) -> Self {
        Self { contract }
    }

    fn contract(&self) -> &'a Contract {
        self.contract
    }
}
