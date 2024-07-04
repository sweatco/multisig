use near_sdk::{
    json_types::{Base64VecU8, U128},
    near, AccountId, Gas, NearToken, PublicKey,
};

pub type MultisigRequestId = u32;

/// Permissions for function call access key.

#[near(serializers=[borsh, json])]
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallPermission {
    pub allowance: Option<U128>,
    pub receiver_id: AccountId,
    pub method_names: Vec<String>,
}

/// Lowest level action that can be performed by the multisig contract.
#[near(serializers=[borsh, json])]
#[derive(Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum MultiSigRequestAction {
    /// Transfers given amount to receiver.
    Transfer { amount: NearToken },
    /// Create a new account.
    CreateAccount,
    /// Deploys contract to receiver's account. Can upgrade given contract as well.
    DeployContract { code: Base64VecU8 },
    /// Adds key, either new key for multisig or full access key to another account.
    AddKey {
        public_key: PublicKey,
        #[serde(skip_serializing_if = "Option::is_none")]
        permission: Option<FunctionCallPermission>,
    },
    /// Deletes key, either one of the keys from multisig or key from another account.
    DeleteKey { public_key: PublicKey },
    /// Call function on behalf of this contract.
    FunctionCall {
        method_name: String,
        args: Base64VecU8,
        deposit: NearToken,
        gas: Gas,
    },
    /// Sets number of confirmations required to authorize requests.
    /// Can not be bundled with any other actions or transactions.
    SetNumConfirmations { num_confirmations: usize },
    /// Sets number of active requests (unconfirmed requests) per access key
    /// Default is 12 unconfirmed requests at a time
    /// The REQUEST_COOLDOWN for requests is 15min
    /// Worst gas attack a malicious keyholder could do is 12 requests every 15min
    SetActiveRequestsLimit { active_requests_limit: u32 },
}

// The request the user makes specifying the receiving account and actions they want to execute (1 tx)
#[near(serializers=[borsh, json])]
#[derive(Debug, Clone, PartialEq)]
pub struct MultiSigRequest {
    pub receiver_id: AccountId,
    pub actions: Vec<MultiSigRequestAction>,
}

// An internal request wrapped with the signer_pk and added timestamp to determine num_requests_pk and prevent against malicious key holder gas attacks
#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct MultiSigRequestWithSigner {
    pub request: MultiSigRequest,
    pub signer_pk: PublicKey,
    pub added_timestamp: u64,
}
