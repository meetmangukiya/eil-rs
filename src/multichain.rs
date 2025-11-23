use crate::types::*;
use alloy::json_abi::JsonAbi;
use alloy::primitives::U256;
use std::collections::HashMap;

pub use crate::types::AddressPerChain;

/// Multi-chain contract abstraction
#[derive(Debug, Clone)]
pub struct MultichainContract {
    /// Contract ABI
    pub abi: JsonAbi,
    /// Deployment addresses per chain
    pub deployments: AddressPerChain,
}

impl MultichainContract {
    /// Create a new multichain contract
    pub fn new(abi: JsonAbi, deployments: AddressPerChain) -> Self {
        Self { abi, deployments }
    }

    /// Get the contract address on a specific chain
    pub fn address_on(&self, chain_id: ChainId) -> Option<Address> {
        self.deployments.get(&chain_id).copied()
    }

    /// Check if contract is deployed on a chain
    pub fn is_deployed_on(&self, chain_id: ChainId) -> bool {
        self.deployments.contains_key(&chain_id)
    }
}

impl MultiChainEntity for MultichainContract {
    fn address_on(&self, chain_id: ChainId) -> Option<Address> {
        self.address_on(chain_id)
    }
}

/// ERC20 token abstraction across multiple chains
#[derive(Debug, Clone)]
pub struct MultichainToken {
    /// Token name/symbol
    pub name: String,
    /// Deployment addresses per chain
    pub deployments: AddressPerChain,
}

impl MultichainToken {
    /// Create a new multichain token
    pub fn new(name: String, deployments: AddressPerChain) -> Self {
        Self { name, deployments }
    }

    /// Get the token address on a specific chain
    pub fn address_on(&self, chain_id: ChainId) -> Option<Address> {
        self.deployments.get(&chain_id).copied()
    }

    /// Check if token is deployed on a chain
    pub fn is_deployed_on(&self, chain_id: ChainId) -> bool {
        self.deployments.contains_key(&chain_id)
    }

    /// Get ERC20 ABI (standard)
    pub fn abi(&self) -> JsonAbi {
        // Standard ERC20 ABI
        serde_json::from_str(ERC20_ABI).expect("Failed to parse ERC20 ABI")
    }
}

impl MultiChainEntity for MultichainToken {
    fn address_on(&self, chain_id: ChainId) -> Option<Address> {
        self.address_on(chain_id)
    }
}

/// Standard ERC20 ABI (minimal)
const ERC20_ABI: &str = r#"[
  {
    "type": "function",
    "name": "balanceOf",
    "stateMutability": "view",
    "inputs": [{"name": "account", "type": "address"}],
    "outputs": [{"name": "", "type": "uint256"}]
  },
  {
    "type": "function",
    "name": "transfer",
    "stateMutability": "nonpayable",
    "inputs": [
      {"name": "to", "type": "address"},
      {"name": "amount", "type": "uint256"}
    ],
    "outputs": [{"name": "", "type": "bool"}]
  },
  {
    "type": "function",
    "name": "approve",
    "stateMutability": "nonpayable",
    "inputs": [
      {"name": "spender", "type": "address"},
      {"name": "amount", "type": "uint256"}
    ],
    "outputs": [{"name": "", "type": "bool"}]
  },
  {
    "type": "function",
    "name": "allowance",
    "stateMutability": "view",
    "inputs": [
      {"name": "owner", "type": "address"},
      {"name": "spender", "type": "address"}
    ],
    "outputs": [{"name": "", "type": "uint256"}]
  },
  {
    "type": "function",
    "name": "decimals",
    "stateMutability": "view",
    "inputs": [],
    "outputs": [{"name": "", "type": "uint8"}]
  },
  {
    "type": "function",
    "name": "symbol",
    "stateMutability": "view",
    "inputs": [],
    "outputs": [{"name": "", "type": "string"}]
  }
]"#;

/// Result of total balance query across chains
#[derive(Debug, Clone)]
pub struct TotalBalanceOfResult {
    /// Balance per chain
    pub per_chain_balance: Vec<(ChainId, U256)>,
    /// Total balance across all chains
    pub total_balance: U256,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_deployments() -> AddressPerChain {
        let mut deployments = HashMap::new();
        deployments.insert(1, "0x0000000000000000000000000000000000000001".parse().unwrap());
        deployments.insert(10, "0x0000000000000000000000000000000000000010".parse().unwrap());
        deployments
    }

    #[test]
    fn test_multichain_contract_new() {
        let deployments = create_test_deployments();
        let abi = serde_json::from_str("[]").unwrap();
        let contract = MultichainContract::new(abi, deployments);

        assert_eq!(contract.deployments.len(), 2);
    }

    #[test]
    fn test_multichain_contract_address_on() {
        let deployments = create_test_deployments();
        let abi = serde_json::from_str("[]").unwrap();
        let contract = MultichainContract::new(abi, deployments);

        assert!(contract.address_on(1).is_some());
        assert!(contract.address_on(10).is_some());
        assert!(contract.address_on(999).is_none());
    }

    #[test]
    fn test_multichain_contract_is_deployed_on() {
        let deployments = create_test_deployments();
        let abi = serde_json::from_str("[]").unwrap();
        let contract = MultichainContract::new(abi, deployments);

        assert!(contract.is_deployed_on(1));
        assert!(contract.is_deployed_on(10));
        assert!(!contract.is_deployed_on(999));
    }

    #[test]
    fn test_multichain_token_new() {
        let deployments = create_test_deployments();
        let token = MultichainToken::new("USDC".to_string(), deployments);

        assert_eq!(token.name, "USDC");
        assert_eq!(token.deployments.len(), 2);
    }

    #[test]
    fn test_multichain_token_address_on() {
        let deployments = create_test_deployments();
        let token = MultichainToken::new("USDC".to_string(), deployments);

        let addr1 = token.address_on(1);
        let addr10 = token.address_on(10);

        assert!(addr1.is_some());
        assert!(addr10.is_some());
        assert_ne!(addr1.unwrap(), addr10.unwrap());
    }

    #[test]
    fn test_multichain_token_is_deployed_on() {
        let deployments = create_test_deployments();
        let token = MultichainToken::new("USDC".to_string(), deployments);

        assert!(token.is_deployed_on(1));
        assert!(token.is_deployed_on(10));
        assert!(!token.is_deployed_on(42161));
    }

    #[test]
    fn test_multichain_token_abi() {
        let deployments = create_test_deployments();
        let token = MultichainToken::new("USDC".to_string(), deployments);

        let abi = token.abi();

        // Should have standard ERC20 functions
        assert!(abi.function("transfer").is_some());
        assert!(abi.function("approve").is_some());
        assert!(abi.function("balanceOf").is_some());
        assert!(abi.function("allowance").is_some());
    }

    #[test]
    fn test_multichain_entity_trait() {
        let deployments = create_test_deployments();
        let token = MultichainToken::new("USDC".to_string(), deployments);

        // Test through MultiChainEntity trait
        let entity: &dyn MultiChainEntity = &token;
        assert!(entity.address_on(1).is_some());
        assert!(entity.address_on(999).is_none());
    }
}
