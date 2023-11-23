use integration_trait::make_integration_version;
use near_sdk::{PromiseOrValue, PublicKey};

use crate::data::{MultiSigRequest, MultisigRequestId};

#[make_integration_version]
pub trait MultisigApi {
    /// Initialize multisig contract.
    /// @params num_confirmations: k of n signatures required to perform operations.
    fn new(num_confirmations: usize) -> Self;

    /// Add request for multisig.
    fn add_request(&mut self, request: MultiSigRequest) -> MultisigRequestId;

    /// Add request for multisig and confirm with the pk that added.
    fn add_request_and_confirm(&mut self, request: MultiSigRequest) -> MultisigRequestId;

    /// Remove given request and associated confirmations.
    fn delete_request(&mut self, request_id: MultisigRequestId) -> MultiSigRequest;

    /// Confirm given request with given signing key.
    /// If with this, there has been enough confirmation, a promise with request will be scheduled.
    fn confirm(&mut self, request_id: MultisigRequestId) -> PromiseOrValue<bool>;
}

#[make_integration_version]
pub trait MultisigView {
    fn get_request(&self, request_id: MultisigRequestId) -> MultiSigRequest;

    fn get_num_requests_pk(&self, public_key: PublicKey) -> u32;

    fn list_request_ids(&self) -> Vec<MultisigRequestId>;

    fn get_confirmations(&self, request_id: MultisigRequestId) -> Vec<PublicKey>;

    fn get_num_confirmations(&self) -> usize;

    fn get_request_nonce(&self) -> u32;
}
