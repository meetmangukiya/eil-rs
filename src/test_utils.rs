//! Test utilities and mock implementations
//! Available in both unit tests and integration tests

use crate::{
    account::{BundlerManager, MultiChainSmartAccount, Signer},
    contract_types::UserOperation,
    types::*,
    Result,
};
use alloy::primitives::U256;
use async_trait::async_trait;
use std::collections::HashMap;

/// Mock signer for testing
pub struct MockSigner {
    pub address: Address,
}

impl MockSigner {
    pub fn new() -> Self {
        Self {
            address: "0x1111111111111111111111111111111111111111"
                .parse()
                .unwrap(),
        }
    }
}

#[async_trait]
impl Signer for MockSigner {
    async fn sign(&self, _hash: &[u8; 32]) -> Result<Hex> {
        // Return a dummy signature
        Ok(Hex::from(vec![0u8; 65]))
    }

    fn address(&self) -> Address {
        self.address
    }
}

/// Mock bundler manager for testing
pub struct MockBundlerManager {
    pub submitted_ops: std::sync::Arc<std::sync::Mutex<Vec<UserOperation>>>,
}

impl MockBundlerManager {
    pub fn new() -> Self {
        Self {
            submitted_ops: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn get_submitted_ops(&self) -> Vec<UserOperation> {
        self.submitted_ops.lock().unwrap().clone()
    }
}

#[async_trait]
impl BundlerManager for MockBundlerManager {
    async fn send_user_operation(
        &self,
        user_op: &UserOperation,
        _entry_point: Address,
    ) -> Result<Hex> {
        self.submitted_ops.lock().unwrap().push(user_op.clone());
        // Return a dummy UserOp hash
        Ok(Hex::from(vec![0xabu8; 32]))
    }

    async fn verify_entry_point(&self, _chain_id: ChainId, _entry_point: Address) -> Result<()> {
        Ok(())
    }
}

/// Mock multi-chain smart account for testing
pub struct MockAccount {
    pub addresses: HashMap<ChainId, Address>,
    pub signer: MockSigner,
    pub bundler: MockBundlerManager,
}

impl MockAccount {
    pub fn new() -> Self {
        let mut addresses = HashMap::new();
        addresses.insert(
            1,
            "0x2222222222222222222222222222222222222222"
                .parse()
                .unwrap(),
        );
        addresses.insert(
            10,
            "0x2222222222222222222222222222222222222222"
                .parse()
                .unwrap(),
        );
        addresses.insert(
            42161,
            "0x2222222222222222222222222222222222222222"
                .parse()
                .unwrap(),
        );

        Self {
            addresses,
            signer: MockSigner::new(),
            bundler: MockBundlerManager::new(),
        }
    }

    pub fn with_chains(chain_ids: Vec<ChainId>) -> Self {
        let mut addresses = HashMap::new();
        let addr: Address = "0x2222222222222222222222222222222222222222"
            .parse()
            .unwrap();

        for chain_id in chain_ids {
            addresses.insert(chain_id, addr);
        }

        Self {
            addresses,
            signer: MockSigner::new(),
            bundler: MockBundlerManager::new(),
        }
    }
}

#[async_trait]
impl MultiChainSmartAccount for MockAccount {
    fn address_on(&self, chain_id: ChainId) -> Result<Address> {
        self.addresses
            .get(&chain_id)
            .copied()
            .ok_or_else(|| crate::EilError::UnsupportedChain(chain_id))
    }

    async fn sign_user_ops(&self, mut user_ops: Vec<UserOperation>) -> Result<Vec<UserOperation>> {
        for user_op in &mut user_ops {
            // Simple dummy signature
            user_op.signature = Hex::from(vec![0xabu8; 65]);
        }
        Ok(user_ops)
    }

    async fn encode_calls(&self, _chain_id: ChainId, _calls: Vec<Call>) -> Result<Hex> {
        // Return dummy encoded calls
        Ok(Hex::from(vec![0u8; 32]))
    }

    async fn send_user_operation(&self, user_op: UserOperation) -> Result<Hex> {
        let entry_point = user_op
            .entry_point_address
            .ok_or_else(|| crate::EilError::Generic("No entry point".into()))?;
        self.bundler.send_user_operation(&user_op, entry_point).await
    }

    async fn verify_bundler_config(
        &self,
        chain_id: ChainId,
        entry_point: Address,
    ) -> Result<()> {
        self.bundler.verify_entry_point(chain_id, entry_point).await
    }

    async fn get_nonce(&self, _chain_id: ChainId) -> Result<U256> {
        Ok(U256::from(0))
    }

    async fn get_factory_args(&self, _chain_id: ChainId) -> Result<(Option<Address>, Option<Hex>)> {
        Ok((None, None))
    }
}

/// Create a test configuration with the specified chains
pub fn create_test_config(chain_ids: Vec<ChainId>) -> crate::config::CrossChainConfig {
    use crate::config::{ChainInfo, CrossChainConfig};

    let chain_infos: Vec<ChainInfo> = chain_ids
        .into_iter()
        .map(|id| ChainInfo {
            chain_id: id,
            rpc_url: format!("https://test-rpc-{}.example.com", id),
            entry_point: "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
                .parse()
                .unwrap(),
            paymaster: "0x0000000000000000000000000000000000000001"
                .parse()
                .unwrap(),
            bundler_url: None,
        })
        .collect();

    CrossChainConfig::new(chain_infos)
}

/// Create a test token deployed on the specified chains
pub fn create_test_token(
    name: &str,
    chain_ids: Vec<ChainId>,
) -> crate::multichain::MultichainToken {
    use crate::multichain::MultichainToken;

    let mut deployments = HashMap::new();
    for (i, chain_id) in chain_ids.iter().enumerate() {
        let addr_str = format!("0x{:040x}", i + 1);
        deployments.insert(*chain_id, addr_str.parse().unwrap());
    }

    MultichainToken::new(name.to_string(), deployments)
}
