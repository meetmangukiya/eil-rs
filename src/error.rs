use thiserror::Error;

/// Result type alias for EIL SDK operations
pub type Result<T> = std::result::Result<T, EilError>;

/// Errors that can occur when using the EIL SDK
#[derive(Error, Debug)]
pub enum EilError {
    /// Chain not supported or configured
    #[error("Chain {0} not supported")]
    UnsupportedChain(u64),

    /// Invalid address for chain
    #[error("Invalid address for chain {chain_id}: {address}")]
    InvalidAddress { chain_id: u64, address: String },

    /// Voucher request not found
    #[error("Voucher request '{0}' not found")]
    VoucherNotFound(String),

    /// Voucher request already exists
    #[error("Voucher request '{0}' already exists")]
    DuplicateVoucher(String),

    /// Voucher not consumed
    #[error("Voucher request '{0}' created on chain {1} not used in any other batch")]
    VoucherNotConsumed(String, u64),

    /// Voucher already used
    #[error("Voucher request '{0}' already used")]
    VoucherAlreadyUsed(String),

    /// Invalid voucher destination
    #[error("Voucher request is for chain {expected}, but batch is for chain {actual}")]
    InvalidVoucherDestination { expected: u64, actual: u64 },

    /// Account not set
    #[error("Must call use_account() before build")]
    AccountNotSet,

    /// Account already set
    #[error("Cannot call use_account() more than once")]
    AccountAlreadySet,

    /// Builder already built
    #[error("CrossChainBuilder already built. Create a new instance to build a new session.")]
    BuilderAlreadyBuilt,

    /// Contract not deployed
    #[error("Contract {name} not deployed on chain {chain_id} at address {address}")]
    ContractNotDeployed {
        name: String,
        chain_id: u64,
        address: String,
    },

    /// Contract function not supported
    #[error("Contract {name} on chain {chain_id} at address {address} not supported: {function} {reason}")]
    ContractNotSupported {
        name: String,
        chain_id: u64,
        address: String,
        function: String,
        reason: String,
    },

    /// No XLPs found
    #[error("No XLPs found on destination chain {0} with enough balance")]
    NoXlpsFound(u64),

    /// Insufficient XLPs
    #[error("Only found {found} XLPs on destination chain {chain_id} with enough balance. Minimum required is {required}")]
    InsufficientXlps {
        found: usize,
        required: usize,
        chain_id: u64,
    },

    /// Invalid runtime variable name
    #[error("Variable name '{0}' is too long, must be max 8 characters")]
    InvalidVariableName(String),

    /// Runtime variable with dynamic arguments
    #[error("SetVarAction('{0}'): call must not be dynamic")]
    DynamicVariableCall(String),

    /// Same chain voucher request
    #[error("destinationChainId must be different than current chainId {0}")]
    SameChainVoucher(u64),

    /// Cannot override paymaster
    #[error("Cannot override paymaster or paymasterData in a batch that uses vouchers")]
    CannotOverridePaymaster,

    /// No voucher for chain
    #[error("No voucher requests found for chain {0}")]
    NoVoucherForChain(u64),

    /// UserOperation not signed
    #[error("All UserOperations must be signed before execution")]
    UserOpNotSigned,

    /// Execution already started
    #[error("execute() already called")]
    ExecutionAlreadyStarted,

    /// Execution timeout
    #[error("Execution timeout after {0} seconds")]
    ExecutionTimeout(u64),

    /// Alloy provider error
    #[error("Alloy provider error: {0}")]
    AlloyProvider(String),

    /// Alloy contract error
    #[error("Alloy contract error: {0}")]
    AlloyContract(String),

    /// Alloy signer error
    #[error("Alloy signer error: {0}")]
    AlloySigner(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Hex decoding error
    #[error("Hex decoding error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    /// Generic error
    #[error("{0}")]
    Generic(String),
}

impl From<String> for EilError {
    fn from(s: String) -> Self {
        EilError::Generic(s)
    }
}

impl From<&str> for EilError {
    fn from(s: &str) -> Self {
        EilError::Generic(s.to_string())
    }
}
