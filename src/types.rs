use alloy::{
    primitives::{Address as AlloyAddress, Bytes, U256},
    rpc::types::eth::BlockNumberOrTag,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Chain ID type
pub type ChainId = u64;

/// Address type (re-export from alloy)
pub type Address = AlloyAddress;

/// Hex data type (re-export from alloy)
pub type Hex = Bytes;

/// Common chain IDs for convenience
#[allow(dead_code)]
pub mod chain_ids {
    use super::ChainId;

    pub const MAINNET: ChainId = 1;
    pub const OPTIMISM: ChainId = 10;
    pub const ARBITRUM: ChainId = 42161;
    pub const BASE: ChainId = 8453;
    pub const POLYGON: ChainId = 137;
}

/// Amount can be either a fixed value or a runtime variable
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Amount {
    /// Fixed amount known at build time
    Fixed(U256),
    /// Runtime variable resolved on-chain
    Runtime(RuntimeVar),
}

impl From<U256> for Amount {
    fn from(value: U256) -> Self {
        Amount::Fixed(value)
    }
}

impl From<u64> for Amount {
    fn from(value: u64) -> Self {
        Amount::Fixed(U256::from(value))
    }
}

impl From<RuntimeVar> for Amount {
    fn from(var: RuntimeVar) -> Self {
        Amount::Runtime(var)
    }
}

/// Runtime variable reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RuntimeVar {
    /// Variable name (max 8 characters)
    pub name: String,
}

impl RuntimeVar {
    /// Create a new runtime variable reference
    pub fn new(name: impl Into<String>) -> crate::Result<Self> {
        let name = name.into();
        if name.len() > 8 {
            return Err(crate::EilError::InvalidVariableName(name));
        }
        Ok(Self { name })
    }
}

/// Multi-chain entity trait - can provide addresses on different chains
pub trait MultiChainEntity {
    /// Get the address on a specific chain
    fn address_on(&self, chain_id: ChainId) -> Option<Address>;
}

/// Helper to convert MultiChainEntity to Address
pub fn to_address(chain_id: ChainId, entity: &dyn MultiChainEntity) -> crate::Result<Address> {
    entity
        .address_on(chain_id)
        .ok_or_else(|| crate::EilError::InvalidAddress {
            chain_id,
            address: "None".to_string(),
        })
}

/// Map of addresses per chain
pub type AddressPerChain = HashMap<ChainId, Address>;

/// Token amount with token reference
#[derive(Debug, Clone)]
pub struct TokenAmount {
    /// Token reference (can be multichain)
    pub token: crate::multichain::MultichainToken,
    /// Amount to transfer
    pub amount: Amount,
    /// Minimum provider deposit (for runtime amounts)
    pub min_provider_deposit: Option<U256>,
}

/// Call structure for contract interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    /// Target contract address
    pub target: Address,
    /// Calldata
    pub data: Hex,
    /// Value to send (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
}

/// Function call with ABI encoding
#[derive(Debug, Clone)]
pub struct FunctionCall {
    /// Target contract address
    pub target: Address,
    /// ABI of the contract
    pub abi: alloy::json_abi::JsonAbi,
    /// Function name
    pub function_name: String,
    /// Function arguments
    pub args: Vec<alloy::dyn_abi::DynSolValue>,
    /// Value to send (optional)
    pub value: Option<U256>,
}

/// Operation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatus {
    /// Pending execution
    Pending,
    /// Currently executing
    Executing,
    /// Execution completed successfully
    Done,
    /// Execution failed
    Failed,
}

/// Block number or tag
pub type BlockNumber = BlockNumberOrTag;

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;

    #[test]
    fn test_amount_from_u256() {
        let amount = Amount::from(U256::from(100));
        match amount {
            Amount::Fixed(val) => assert_eq!(val, U256::from(100)),
            Amount::Runtime(_) => panic!("Expected Fixed amount"),
        }
    }

    #[test]
    fn test_amount_from_u64() {
        let amount = Amount::from(42u64);
        match amount {
            Amount::Fixed(val) => assert_eq!(val, U256::from(42)),
            Amount::Runtime(_) => panic!("Expected Fixed amount"),
        }
    }

    #[test]
    fn test_runtime_var_valid() {
        let var = RuntimeVar::new("myvar").unwrap();
        assert_eq!(var.name, "myvar");
    }

    #[test]
    fn test_runtime_var_max_length() {
        let var = RuntimeVar::new("12345678").unwrap();
        assert_eq!(var.name, "12345678");
    }

    #[test]
    fn test_runtime_var_too_long() {
        let result = RuntimeVar::new("123456789");
        assert!(result.is_err());
        match result {
            Err(crate::EilError::InvalidVariableName(name)) => {
                assert_eq!(name, "123456789");
            }
            _ => panic!("Expected InvalidVariableName error"),
        }
    }

    #[test]
    fn test_chain_ids_constants() {
        use crate::types::chain_ids::*;
        assert_eq!(MAINNET, 1);
        assert_eq!(OPTIMISM, 10);
        assert_eq!(ARBITRUM, 42161);
        assert_eq!(BASE, 8453);
        assert_eq!(POLYGON, 137);
    }
}
