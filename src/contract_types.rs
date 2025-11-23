use crate::types::*;
use alloy::primitives::U256;
use serde::{Deserialize, Serialize};

/// ERC-4337 UserOperation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperation {
    /// Account sending the operation
    pub sender: Address,
    /// Anti-replay nonce
    pub nonce: U256,
    /// Account factory address (for deployment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factory: Option<Address>,
    /// Factory data (for deployment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factory_data: Option<Hex>,
    /// Encoded calls to execute
    pub call_data: Hex,
    /// Gas limit for the execution phase
    pub call_gas_limit: U256,
    /// Gas limit for the verification phase
    pub verification_gas_limit: U256,
    /// Gas overhead for pre-verification
    pub pre_verification_gas: U256,
    /// Maximum fee per gas
    pub max_fee_per_gas: U256,
    /// Maximum priority fee per gas
    pub max_priority_fee_per_gas: U256,
    /// Paymaster address (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paymaster: Option<Address>,
    /// Paymaster verification gas limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paymaster_verification_gas_limit: Option<U256>,
    /// Paymaster post-operation gas limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paymaster_post_op_gas_limit: Option<U256>,
    /// Paymaster-specific data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paymaster_data: Option<Hex>,
    /// Paymaster signature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paymaster_signature: Option<Hex>,
    /// Account signature
    pub signature: Hex,
    /// Chain ID (affects hash via EIP-712 domain)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<ChainId>,
    /// EntryPoint address (affects hash via EIP-712 domain)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_point_address: Option<Address>,
}

/// Asset (ERC20 token with amount)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    /// ERC20 token address
    pub erc20_token: Address,
    /// Amount
    pub amount: U256,
}

/// Fee rule for atomic swaps
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtomicSwapFeeRule {
    /// Starting fee percentage (numerator out of 10_000)
    pub start_fee_percent_numerator: U256,
    /// Maximum fee percentage (numerator out of 10_000)
    pub max_fee_percent_numerator: U256,
    /// Fee increase per second (numerator out of 10_000)
    pub fee_increase_per_second: U256,
    /// Unspent voucher fee (numerator out of 10_000)
    pub unspent_voucher_fee: U256,
}

/// Source chain component of a voucher request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSwapComponent {
    /// Source chain ID
    pub chain_id: ChainId,
    /// Sender address on source chain
    pub sender: Address,
    /// Paymaster address on source chain
    pub paymaster: Address,
    /// Assets to transfer
    pub assets: Vec<Asset>,
    /// Fee rule for the swap
    pub fee_rule: AtomicSwapFeeRule,
    /// Sender nonce for voucher uniqueness
    pub sender_nonce: U256,
    /// Allowed XLP addresses
    pub allowed_xlps: Vec<Address>,
}

/// Destination chain component of a voucher request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DestinationSwapComponent {
    /// Destination chain ID
    pub chain_id: ChainId,
    /// Sender address on destination chain
    pub sender: Address,
    /// Paymaster address on destination chain
    pub paymaster: Address,
    /// Assets to receive
    pub assets: Vec<Asset>,
    /// Maximum UserOp cost
    pub max_user_op_cost: U256,
    /// Expiration timestamp
    pub expires_at: U256,
}

/// Voucher request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoucherRequest {
    /// Source chain component
    pub origination: SourceSwapComponent,
    /// Destination chain component
    pub destination: DestinationSwapComponent,
}

/// Signed voucher (returned by XLP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voucher {
    /// Voucher request
    pub request: VoucherRequest,
    /// XLP signature
    pub signature: Hex,
}

/// Session data for ephemeral signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session data payload
    pub data: Hex,
    /// Ephemeral signature
    pub ephemeral_signature: Hex,
}

/// XLP (Cross-chain Liquidity Provider) entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XlpEntry {
    /// XLP address on L1
    pub l1_xlp_address: Address,
    /// XLP address on L2
    pub l2_xlp_address: Address,
    /// Bond amount
    pub bond: U256,
}

/// Single chain batch information
#[derive(Debug, Clone)]
pub struct SingleChainBatch {
    /// UserOperation for this batch
    pub user_op: UserOperation,
    /// UserOperation hash
    pub user_op_hash: Hex,
    /// Chain ID for this batch
    pub chain_id: ChainId,
    /// Input voucher requests (vouchers consumed by this batch)
    pub input_voucher_requests: Vec<SdkVoucherRequest>,
    /// Output voucher requests (vouchers created by this batch)
    pub out_voucher_requests: Vec<SdkVoucherRequest>,
}

/// SDK-level voucher request (before conversion to contract VoucherRequest)
#[derive(Debug, Clone)]
pub struct SdkVoucherRequest {
    /// Reference ID for this voucher
    pub ref_id: String,
    /// Source chain ID (optional, defaults to batch chain)
    pub source_chain_id: Option<ChainId>,
    /// Destination chain ID
    pub destination_chain_id: ChainId,
    /// Tokens to transfer
    pub tokens: Vec<TokenAmount>,
    /// Target address on destination (optional, defaults to sender)
    pub target: Option<Address>,
}

/// Batch status information during execution
#[derive(Debug, Clone)]
pub struct BatchStatusInfo {
    /// Index in the batch array
    pub index: usize,
    /// The batch being executed
    pub batch: SingleChainBatch,
    /// Current status
    pub status: OperationStatus,
    /// Vouchers collected for this batch
    pub vouchers: std::collections::HashMap<String, Voucher>,
    /// Request IDs for vouchers
    pub request_ids: Option<Vec<Hex>>,
    /// Transaction hash (once executed)
    pub tx_hash: Option<Hex>,
    /// Revert reason (if failed)
    pub revert_reason: Option<String>,
}
