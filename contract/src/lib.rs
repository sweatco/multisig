use std::collections::HashSet;

use multisig_model::{
    MultiSigRequest, MultiSigRequestAction, MultiSigRequestWithSigner, MultisigApi, MultisigRequestId, MultisigView,
};
use near_sdk::{
    collections::UnorderedMap, env, near, near_bindgen, AccountId, NearToken, PanicOnDefault, Promise, PromiseOrValue,
    PublicKey,
};

/// Unlimited allowance for multisig keys.
const DEFAULT_ALLOWANCE: NearToken = NearToken::from_yoctonear(0);

// Request cooldown period (time before a request can be deleted)
const REQUEST_COOLDOWN: u64 = 900_000_000_000;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct MultiSigContract {
    num_confirmations: usize,
    request_nonce: MultisigRequestId,
    requests: UnorderedMap<MultisigRequestId, MultiSigRequestWithSigner>,
    confirmations: UnorderedMap<MultisigRequestId, HashSet<PublicKey>>,
    num_requests_pk: UnorderedMap<PublicKey, u32>,
    // per key
    active_requests_limit: u32,
}

#[near_bindgen]
impl MultisigApi for MultiSigContract {
    /// Initialize multisig contract.
    /// @params num_confirmations: k of n signatures required to perform operations.
    #[init]
    fn new(num_confirmations: usize) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            num_confirmations,
            request_nonce: 0,
            requests: UnorderedMap::new(b"r".to_vec()),
            confirmations: UnorderedMap::new(b"c".to_vec()),
            num_requests_pk: UnorderedMap::new(b"k".to_vec()),
            active_requests_limit: 12,
        }
    }

    /// Add request for multisig.
    fn add_request(&mut self, request: MultiSigRequest) -> MultisigRequestId {
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Predecessor account must much current account"
        );
        // track how many requests this key has made
        let num_requests = self.num_requests_pk.get(&env::signer_account_pk()).unwrap_or(0) + 1;
        assert!(
            num_requests <= self.active_requests_limit,
            "Account has too many active requests. Confirm or delete some."
        );
        self.num_requests_pk.insert(&env::signer_account_pk(), &num_requests);
        // add the request
        let request_added = MultiSigRequestWithSigner {
            signer_pk: env::signer_account_pk(),
            added_timestamp: env::block_timestamp(),
            request,
        };
        self.requests.insert(&self.request_nonce, &request_added);
        let confirmations = HashSet::new();
        self.confirmations.insert(&self.request_nonce, &confirmations);
        self.request_nonce += 1;
        self.request_nonce - 1
    }

    /// Add request for multisig and confirm with the pk that added.
    fn add_request_and_confirm(&mut self, request: MultiSigRequest) -> MultisigRequestId {
        let request_id = self.add_request(request);
        self.confirm(request_id);
        request_id
    }

    /// Remove given request and associated confirmations.
    fn delete_request(&mut self, request_id: MultisigRequestId) -> MultiSigRequest {
        self.assert_valid_request(request_id);
        let request_with_signer = self.requests.get(&request_id).expect("No such request");
        // can't delete requests before 15min
        assert!(
            env::block_timestamp() > request_with_signer.added_timestamp + REQUEST_COOLDOWN,
            "Request cannot be deleted immediately after creation."
        );
        self.remove_request(request_id)
    }

    /// Confirm given request with given signing key.
    /// If with this, there has been enough confirmation, a promise with request will be scheduled.
    fn confirm(&mut self, request_id: MultisigRequestId) -> PromiseOrValue<()> {
        self.assert_valid_request(request_id);
        let mut confirmations = self.confirmations.get(&request_id).unwrap();
        assert!(
            !confirmations.contains(&env::signer_account_pk()),
            "Already confirmed this request with this key"
        );
        if confirmations.len() + 1 >= self.num_confirmations {
            let request = self.remove_request(request_id);
            /********************************
            NOTE: If the tx execution fails for any reason, the request and confirmations are removed already, so the client has to start all over
            ********************************/
            self.execute_request(request)
        } else {
            confirmations.insert(env::signer_account_pk());
            self.confirmations.insert(&request_id, &confirmations);
            PromiseOrValue::Value(())
        }
    }
}

#[near_bindgen]
impl MultisigView for MultiSigContract {
    fn get_request(&self, request_id: MultisigRequestId) -> MultiSigRequest {
        (self.requests.get(&request_id).expect("No such request")).request
    }

    fn get_num_requests_pk(&self, public_key: PublicKey) -> u32 {
        self.num_requests_pk.get(&public_key).unwrap_or(0)
    }

    fn list_request_ids(&self) -> Vec<MultisigRequestId> {
        self.requests.keys().collect()
    }

    fn get_confirmations(&self, request_id: MultisigRequestId) -> Vec<PublicKey> {
        self.confirmations
            .get(&request_id)
            .expect("No such request")
            .into_iter()
            .collect()
    }

    fn get_num_confirmations(&self) -> usize {
        self.num_confirmations
    }

    fn get_request_nonce(&self) -> u32 {
        self.request_nonce
    }
}

impl MultiSigContract {
    /********************************
    Helper methods
    ********************************/

    fn execute_request(&mut self, request: MultiSigRequest) -> PromiseOrValue<()> {
        let mut promise = Promise::new(request.receiver_id.clone());
        let receiver_id = request.receiver_id.clone();
        let num_actions = request.actions.len();
        for action in request.actions {
            promise = match action {
                MultiSigRequestAction::Transfer { amount } => promise.transfer(amount),
                MultiSigRequestAction::CreateAccount => promise.create_account(),
                MultiSigRequestAction::DeployContract { code } => promise.deploy_contract(code.into()),
                MultiSigRequestAction::AddKey { public_key, permission } => {
                    assert_self_request(receiver_id.clone());
                    if let Some(permission) = permission {
                        // TODO:
                        #[allow(deprecated)]
                        promise.add_access_key(
                            public_key,
                            permission
                                .allowance
                                .map_or(DEFAULT_ALLOWANCE, |allowance| NearToken::from_yoctonear(allowance.0)),
                            permission.receiver_id,
                            permission.method_names.join(","),
                        )
                    } else {
                        // wallet UI should warn user if receiver_id == env::current_account_id(), adding FAK will render multisig useless
                        promise.add_full_access_key(public_key)
                    }
                }
                MultiSigRequestAction::DeleteKey { public_key } => {
                    assert_self_request(receiver_id.clone());
                    let pk: PublicKey = public_key;
                    // delete outstanding requests by public_key
                    let request_ids: Vec<u32> = self
                        .requests
                        .iter()
                        .filter(|(_k, r)| r.signer_pk == pk)
                        .map(|(k, _r)| k)
                        .collect();
                    for request_id in request_ids {
                        // remove confirmations for this request
                        self.confirmations.remove(&request_id);
                        self.requests.remove(&request_id);
                    }
                    // remove num_requests_pk entry for public_key
                    self.num_requests_pk.remove(&pk);
                    promise.delete_key(pk)
                }
                MultiSigRequestAction::FunctionCall {
                    method_name,
                    args,
                    deposit,
                    gas,
                } => {
                    env::log_str(&format!("method_name: {method_name}"));
                    env::log_str(&format!("deposit: {deposit:?}"));
                    env::log_str(&format!("gas: {gas:?}"));
                    env::log_str(&format!("args.0.len(): {}", args.0.len()));

                    promise.function_call(method_name, args.into(), deposit, gas)
                }
                // the following methods must be a single action
                MultiSigRequestAction::SetNumConfirmations { num_confirmations } => {
                    assert_one_action_only(receiver_id, num_actions);
                    self.num_confirmations = num_confirmations;
                    return PromiseOrValue::Value(());
                }
                MultiSigRequestAction::SetActiveRequestsLimit { active_requests_limit } => {
                    assert_one_action_only(receiver_id, num_actions);
                    self.active_requests_limit = active_requests_limit;
                    return PromiseOrValue::Value(());
                }
            };
        }
        promise.into()
    }

    // removes request, removes confirmations and reduces num_requests_pk - used in delete, delete_key, and confirm
    fn remove_request(&mut self, request_id: MultisigRequestId) -> MultiSigRequest {
        // remove confirmations for this request
        self.confirmations.remove(&request_id);
        // remove the original request
        let request_with_signer = self
            .requests
            .remove(&request_id)
            .expect("Failed to remove existing element");
        // decrement num_requests for original request signer
        let original_signer_pk = request_with_signer.signer_pk;
        let mut num_requests = self.num_requests_pk.get(&original_signer_pk).unwrap_or(0);
        num_requests = num_requests.saturating_sub(1);
        self.num_requests_pk.insert(&original_signer_pk, &num_requests);
        // return request
        request_with_signer.request
    }
    // Prevents access to calling requests and make sure request_id is valid - used in delete and confirm
    fn assert_valid_request(&mut self, request_id: MultisigRequestId) {
        // request must come from key added to contract account
        assert_eq!(
            env::current_account_id(),
            env::predecessor_account_id(),
            "Predecessor account must much current account"
        );
        // request must exist
        assert!(
            self.requests.get(&request_id).is_some(),
            "No such request: either wrong number or already confirmed"
        );
        // request must have
        assert!(
            self.confirmations.get(&request_id).is_some(),
            "Internal error: confirmations mismatch requests"
        );
    }
}

// Prevents request from approving tx on another account
fn assert_self_request(receiver_id: AccountId) {
    assert_eq!(
        receiver_id,
        env::current_account_id(),
        "This method only works when receiver_id is equal to current_account_id"
    );
}
// Prevents a request from being bundled with other actions
fn assert_one_action_only(receiver_id: AccountId, num_actions: usize) {
    assert_self_request(receiver_id);
    assert_eq!(num_actions, 1, "This method should be a separate request");
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use near_sdk::{
        test_utils::test_env::{alice, bob},
        testing_env, AccountId, BlockHeight, EpochHeight, Gas, VMContext,
    };

    use super::*;

    pub struct VMContextBuilder {
        context: VMContext,
    }

    impl VMContextBuilder {
        pub fn new() -> Self {
            Self {
                context: VMContext {
                    current_account_id: AccountId::from_str("current_account_id").unwrap(),
                    signer_account_id: AccountId::from_str("signer_account_id").unwrap(),
                    signer_account_pk: PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
                    predecessor_account_id: AccountId::from_str("predecessor_account_id").unwrap(),
                    input: vec![],
                    epoch_height: 0,
                    block_index: 0,
                    block_timestamp: 0,
                    account_balance: NearToken::from_yoctonear(0),
                    account_locked_balance: NearToken::from_yoctonear(0),
                    storage_usage: 10u64.pow(6),
                    attached_deposit: NearToken::from_yoctonear(0),
                    prepaid_gas: Gas::from_gas(10u64.pow(18)),
                    random_seed: [
                        1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 8, 7, 6, 5, 3,
                    ],
                    view_config: None,
                    output_data_receivers: vec![],
                },
            }
        }

        pub fn current_account_id(mut self, account_id: AccountId) -> Self {
            self.context.current_account_id = account_id;
            self
        }

        pub fn block_timestamp(mut self, time: u64) -> Self {
            self.context.block_timestamp = time;
            self
        }

        #[allow(dead_code)]
        pub fn signer_account_id(mut self, account_id: AccountId) -> Self {
            self.context.signer_account_id = account_id;
            self
        }

        pub fn signer_account_pk(mut self, signer_account_pk: PublicKey) -> Self {
            self.context.signer_account_pk = signer_account_pk;
            self
        }

        pub fn predecessor_account_id(mut self, account_id: AccountId) -> Self {
            self.context.predecessor_account_id = account_id;
            self
        }

        #[allow(dead_code)]
        pub fn block_index(mut self, block_index: BlockHeight) -> Self {
            self.context.block_index = block_index;
            self
        }

        #[allow(dead_code)]
        pub fn epoch_height(mut self, epoch_height: EpochHeight) -> Self {
            self.context.epoch_height = epoch_height;
            self
        }

        #[allow(dead_code)]
        pub fn attached_deposit(mut self, amount: NearToken) -> Self {
            self.context.attached_deposit = amount;
            self
        }

        pub fn account_balance(mut self, amount: NearToken) -> Self {
            self.context.account_balance = amount;
            self
        }

        #[allow(dead_code)]
        pub fn account_locked_balance(mut self, amount: NearToken) -> Self {
            self.context.account_locked_balance = amount;
            self
        }

        pub fn finish(self) -> VMContext {
            self.context
        }
    }

    fn context_with_key(key: PublicKey, amount: NearToken) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(alice())
            .predecessor_account_id(alice())
            .signer_account_id(alice())
            .signer_account_pk(key)
            .account_balance(amount)
            .finish()
    }

    fn context_with_key_future(key: PublicKey, amount: NearToken) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(alice())
            .block_timestamp(REQUEST_COOLDOWN + 1)
            .predecessor_account_id(alice())
            .signer_account_id(alice())
            .signer_account_pk(key)
            .account_balance(amount)
            .finish()
    }

    #[test]
    fn test_multi_3_of_n() {
        let amount = NearToken::from_yoctonear(1_000);
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(3);
        let request = MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer { amount: amount.into() }],
        };
        let request_id = c.add_request(request.clone());
        assert_eq!(c.get_request(request_id), request);
        assert_eq!(c.list_request_ids(), vec![request_id]);
        c.confirm(request_id);
        assert_eq!(c.requests.len(), 1);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 1);
        testing_env!(context_with_key(
            PublicKey::from_str("HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R").unwrap(),
            amount
        ));
        c.confirm(request_id);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 2);
        assert_eq!(c.get_confirmations(request_id).len(), 2);
        testing_env!(context_with_key(
            PublicKey::from_str("2EfbwnQHPBWQKbNczLiVznFghh9qs716QT71zN6L1D95").unwrap(),
            amount
        ));
        c.confirm(request_id);
        // TODO: confirm that funds were transferred out via promise.
        assert_eq!(c.requests.len(), 0);
    }

    #[test]
    fn test_multi_add_request_and_confirm() {
        let amount = NearToken::from_yoctonear(1_000);
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(3);
        let request = MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer { amount: amount.into() }],
        };
        let request_id = c.add_request_and_confirm(request.clone());
        assert_eq!(c.get_request(request_id), request);
        assert_eq!(c.list_request_ids(), vec![request_id]);
        // c.confirm(request_id);
        assert_eq!(c.requests.len(), 1);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 1);
        testing_env!(context_with_key(
            PublicKey::from_str("HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R").unwrap(),
            amount
        ));
        c.confirm(request_id);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 2);
        assert_eq!(c.get_confirmations(request_id).len(), 2);
        testing_env!(context_with_key(
            PublicKey::from_str("2EfbwnQHPBWQKbNczLiVznFghh9qs716QT71zN6L1D95").unwrap(),
            amount
        ));
        c.confirm(request_id);
        // TODO: confirm that funds were transferred out via promise.
        assert_eq!(c.requests.len(), 0);
    }

    #[test]
    fn add_key_delete_key_storage_cleared() {
        let amount = NearToken::from_yoctonear(1_000);
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(1);
        let new_key = PublicKey::from_str("HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R").unwrap();
        // vm current_account_id is alice, receiver_id must be alice
        let request = MultiSigRequest {
            receiver_id: alice(),
            actions: vec![MultiSigRequestAction::AddKey {
                public_key: new_key.clone(),
                permission: None,
            }],
        };
        // make request
        c.add_request_and_confirm(request.clone());
        // should be empty now
        assert_eq!(c.requests.len(), 0);
        // switch accounts
        testing_env!(context_with_key(
            PublicKey::from_str("HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R").unwrap(),
            amount
        ));
        let request2 = MultiSigRequest {
            receiver_id: alice(),
            actions: vec![MultiSigRequestAction::Transfer { amount: amount.into() }],
        };
        // make request but don't confirm
        c.add_request(request2.clone());
        // should have 1 request now
        assert_eq!(c.requests.len(), 1);
        assert_eq!(c.get_num_requests_pk(new_key.clone()), 1);
        // self delete key
        let request3 = MultiSigRequest {
            receiver_id: alice(),
            actions: vec![MultiSigRequestAction::DeleteKey {
                public_key: new_key.clone(),
            }],
        };
        // make request and confirm
        c.add_request_and_confirm(request3.clone());
        // should be empty now
        assert_eq!(c.requests.len(), 0);
        assert_eq!(c.get_num_requests_pk(new_key.clone()), 0);
    }

    #[test]
    #[should_panic]
    fn test_panics_add_key_different_account() {
        let amount = NearToken::from_yoctonear(1_000);
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(1);
        let new_key = PublicKey::from_str("HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R").unwrap();
        // vm current_account_id is alice, receiver_id must be alice
        let request = MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::AddKey {
                public_key: new_key.clone(),
                permission: None,
            }],
        };
        // make request
        c.add_request_and_confirm(request);
    }

    #[test]
    fn test_change_num_confirmations() {
        let amount = NearToken::from_yoctonear(1_000);
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(1);
        let request_id = c.add_request(MultiSigRequest {
            receiver_id: alice(),
            actions: vec![MultiSigRequestAction::SetNumConfirmations { num_confirmations: 2 }],
        });
        c.confirm(request_id);
        assert_eq!(c.num_confirmations, 2);
    }

    #[test]
    #[should_panic]
    fn test_panics_on_second_confirm() {
        let amount = NearToken::from_yoctonear(1_000);
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(3);
        let request_id = c.add_request(MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer { amount: amount.into() }],
        });
        assert_eq!(c.requests.len(), 1);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 0);
        c.confirm(request_id);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 1);
        c.confirm(request_id);
    }

    #[test]
    #[should_panic]
    fn test_panics_delete_request() {
        let amount = NearToken::from_yoctonear(1_000);
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(3);
        let request_id = c.add_request(MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer { amount: amount.into() }],
        });
        c.delete_request(request_id);
        assert_eq!(c.requests.len(), 0);
        assert_eq!(c.confirmations.len(), 0);
    }

    #[test]
    fn test_delete_request_future() {
        let amount = NearToken::from_yoctonear(1_000);
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(3);
        let request_id = c.add_request(MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer { amount: amount.into() }],
        });
        testing_env!(context_with_key_future(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        c.delete_request(request_id);
        assert_eq!(c.requests.len(), 0);
        assert_eq!(c.confirmations.len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_delete_request_panic_wrong_key() {
        let amount = NearToken::from_yoctonear(1_000);
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(3);
        let request_id = c.add_request(MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer { amount: amount.into() }],
        });
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        c.delete_request(request_id);
    }

    #[test]
    #[should_panic]
    fn test_too_many_requests() {
        let amount = NearToken::from_yoctonear(1_000);
        testing_env!(context_with_key(
            PublicKey::from_str("Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(3);
        for _i in 0..16 {
            c.add_request(MultiSigRequest {
                receiver_id: bob(),
                actions: vec![MultiSigRequestAction::Transfer { amount: amount.into() }],
            });
        }
    }
}
