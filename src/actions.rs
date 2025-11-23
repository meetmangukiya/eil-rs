use crate::{
    contract_types::SdkVoucherRequest,
    multichain::MultichainToken,
    types::*,
    Result,
};
use async_trait::async_trait;

/// Base trait for all actions
/// An action represents a single operation that can be executed on-chain
#[async_trait]
pub trait Action: Send + Sync {
    /// Encode this action as an array of Call objects
    /// The batch parameter provides context like chain ID
    async fn encode_call(&self, batch: &crate::builder::BatchBuilder) -> Result<Vec<Call>>;
}

/// Transfer ERC20 tokens
#[derive(Debug, Clone)]
pub struct TransferAction {
    /// Token to transfer
    pub token: MultichainToken,
    /// Recipient address
    pub recipient: Address,
    /// Amount to transfer (can be runtime variable)
    pub amount: Amount,
}

#[async_trait]
impl Action for TransferAction {
    async fn encode_call(&self, batch: &crate::builder::BatchBuilder) -> Result<Vec<Call>> {
        let token_address = self
            .token
            .address_on(batch.chain_id())
            .ok_or_else(|| crate::EilError::InvalidAddress {
                chain_id: batch.chain_id(),
                address: format!("Token {} not deployed", self.token.name),
            })?;

        // Encode transfer function call
        let abi = self.token.abi();
        let functions = abi
            .function("transfer")
            .ok_or_else(|| crate::EilError::Generic("transfer function not found".into()))?;

        let function = functions
            .first()
            .ok_or_else(|| crate::EilError::Generic("transfer function not found".into()))?;

        // Encode based on whether amount is fixed or runtime
        let data = match &self.amount {
            Amount::Fixed(amount) => {
                use alloy::dyn_abi::DynSolValue;
                let args = vec![
                    DynSolValue::Address(self.recipient),
                    DynSolValue::Uint(*amount, 256),
                ];

                // Encode function call: selector + encoded args
                let mut encoded = function.selector().to_vec();

                // Encode args as a tuple
                let tuple = alloy::dyn_abi::DynSolValue::Tuple(args);
                let encoded_args = tuple.abi_encode();
                // Skip the first 32 bytes (offset for dynamic tuple) if needed
                // For simple types like (address, uint256), we can use it directly
                encoded.extend_from_slice(&encoded_args);
                encoded
            }
            Amount::Runtime(var) => {
                // For runtime variables, we'll need special encoding
                // This will be handled by the runtime vars system
                return Err(crate::EilError::Generic(
                    "Runtime variables not yet implemented in encode".into(),
                ));
            }
        };

        Ok(vec![Call {
            target: token_address,
            data: data.into(),
            value: None,
        }])
    }
}

/// Approve ERC20 token spending
#[derive(Debug, Clone)]
pub struct ApproveAction {
    /// Token to approve
    pub token: MultichainToken,
    /// Spender address
    pub spender: Address,
    /// Amount to approve (can be runtime variable)
    pub value: Amount,
}

#[async_trait]
impl Action for ApproveAction {
    async fn encode_call(&self, batch: &crate::builder::BatchBuilder) -> Result<Vec<Call>> {
        let token_address = self
            .token
            .address_on(batch.chain_id())
            .ok_or_else(|| crate::EilError::InvalidAddress {
                chain_id: batch.chain_id(),
                address: format!("Token {} not deployed", self.token.name),
            })?;

        let abi = self.token.abi();
        let functions = abi
            .function("approve")
            .ok_or_else(|| crate::EilError::Generic("approve function not found".into()))?;

        let function = functions
            .first()
            .ok_or_else(|| crate::EilError::Generic("approve function not found".into()))?;

        let data = match &self.value {
            Amount::Fixed(amount) => {
                use alloy::dyn_abi::DynSolValue;
                let args = vec![
                    DynSolValue::Address(self.spender),
                    DynSolValue::Uint(*amount, 256),
                ];

                // Encode function call: selector + encoded args
                let mut encoded = function.selector().to_vec();

                // Encode args as a tuple
                let tuple = alloy::dyn_abi::DynSolValue::Tuple(args);
                let encoded_args = tuple.abi_encode();
                // Skip the first 32 bytes (offset for dynamic tuple) if needed
                // For simple types like (address, uint256), we can use it directly
                encoded.extend_from_slice(&encoded_args);
                encoded
            }
            Amount::Runtime(_var) => {
                return Err(crate::EilError::Generic(
                    "Runtime variables not yet implemented in encode".into(),
                ));
            }
        };

        Ok(vec![Call {
            target: token_address,
            data: data.into(),
            value: None,
        }])
    }
}

/// Generic function call action
#[derive(Debug, Clone)]
pub struct FunctionCallAction {
    /// Call specification
    pub call: FunctionCall,
}

#[async_trait]
impl Action for FunctionCallAction {
    async fn encode_call(&self, batch: &crate::builder::BatchBuilder) -> Result<Vec<Call>> {
        let functions = self
            .call
            .abi
            .function(&self.call.function_name)
            .ok_or_else(|| {
                crate::EilError::Generic(format!(
                    "Function {} not found",
                    self.call.function_name
                ))
            })?;

        let function = functions
            .first()
            .ok_or_else(|| {
                crate::EilError::Generic(format!(
                    "Function {} not found",
                    self.call.function_name
                ))
            })?;

        // Encode function call: selector + encoded args
        let mut data = function.selector().to_vec();

        // Encode args as a tuple
        let tuple = alloy::dyn_abi::DynSolValue::Tuple(self.call.args.clone());
        let encoded_args = tuple.abi_encode();
        data.extend_from_slice(&encoded_args);

        // Validate target address exists on this chain
        if !is_valid_address(self.call.target) {
            return Err(crate::EilError::InvalidAddress {
                chain_id: batch.chain_id(),
                address: format!(
                    "Calling '{}' on contract with no address on chain {}",
                    self.call.function_name,
                    batch.chain_id()
                ),
            });
        }

        Ok(vec![Call {
            target: self.call.target,
            data: data.into(),
            value: self.call.value,
        }])
    }
}

/// Voucher request action
#[derive(Debug, Clone)]
pub struct VoucherRequestAction {
    /// Voucher request specification
    pub voucher_request: SdkVoucherRequest,
}

#[async_trait]
impl Action for VoucherRequestAction {
    async fn encode_call(&self, _batch: &crate::builder::BatchBuilder) -> Result<Vec<Call>> {
        // Voucher requests don't generate calls directly
        // They are processed separately during batch building
        Ok(vec![])
    }
}

/// Set a runtime variable from a function call result
#[derive(Debug, Clone)]
pub struct SetVarAction {
    /// Variable name (max 8 characters)
    pub var_name: String,
    /// Function call to execute and store result
    pub call: FunctionCall,
}

impl SetVarAction {
    /// Create a new SetVarAction
    pub fn new(var_name: impl Into<String>, call: FunctionCall) -> Result<Self> {
        let var_name = var_name.into();
        if var_name.len() > 8 {
            return Err(crate::EilError::InvalidVariableName(var_name));
        }
        Ok(Self { var_name, call })
    }
}

#[async_trait]
impl Action for SetVarAction {
    async fn encode_call(&self, _batch: &crate::builder::BatchBuilder) -> Result<Vec<Call>> {
        // Runtime variable setting requires special encoding through RuntimeVarsHelper
        // This will be implemented in the runtime variables module
        Err(crate::EilError::Generic(
            "SetVarAction encoding not yet implemented".into(),
        ))
    }
}

/// Helper function to validate an address
fn is_valid_address(address: Address) -> bool {
    !address.is_zero()
}
