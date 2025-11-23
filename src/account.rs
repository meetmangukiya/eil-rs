use crate::{contract_types::UserOperation, types::*, Result};
use async_trait::async_trait;

/// Multi-chain smart account trait
/// Provides account abstraction across multiple chains
#[async_trait]
pub trait MultiChainSmartAccount: Send + Sync {
    /// Get the account address on a specific chain
    fn address_on(&self, chain_id: ChainId) -> Result<Address>;

    /// Sign multiple UserOperations for cross-chain execution
    /// Returns the signed UserOperations with signatures filled in
    async fn sign_user_ops(&self, user_ops: Vec<UserOperation>) -> Result<Vec<UserOperation>>;

    /// Encode calls for execution on a specific chain
    /// Converts an array of Call objects into callData hex
    async fn encode_calls(&self, chain_id: ChainId, calls: Vec<Call>) -> Result<Hex>;

    /// Encode static calls (no runtime variables)
    async fn encode_static_calls(&self, chain_id: ChainId, calls: Vec<Call>) -> Result<Hex> {
        // Default implementation same as encode_calls
        self.encode_calls(chain_id, calls).await
    }

    /// Send a UserOperation to the bundler for execution
    async fn send_user_operation(&self, user_op: UserOperation) -> Result<Hex>;

    /// Verify bundler configuration is valid for a chain
    async fn verify_bundler_config(&self, chain_id: ChainId, entry_point: Address)
        -> Result<()>;

    /// Get nonce for the account on a specific chain
    async fn get_nonce(&self, chain_id: ChainId) -> Result<alloy::primitives::U256>;

    /// Get factory args for account deployment (if not deployed)
    async fn get_factory_args(&self, chain_id: ChainId) -> Result<(Option<Address>, Option<Hex>)>;
}

impl MultiChainEntity for dyn MultiChainSmartAccount {
    fn address_on(&self, chain_id: ChainId) -> Option<Address> {
        MultiChainSmartAccount::address_on(self, chain_id).ok()
    }
}

/// Base implementation helper for MultiChainSmartAccount
/// Provides common functionality for smart account implementations
pub struct BaseMultichainSmartAccount {
    /// Addresses per chain
    pub addresses: std::collections::HashMap<ChainId, Address>,
    /// Signer (for signing UserOps)
    pub signer: Box<dyn Signer>,
    /// Bundler manager (for sending UserOps)
    pub bundler_manager: Box<dyn BundlerManager>,
}

/// Signer trait for signing UserOperations
#[async_trait]
pub trait Signer: Send + Sync {
    /// Sign a UserOperation hash
    async fn sign(&self, hash: &[u8; 32]) -> Result<Hex>;

    /// Get the signer address
    fn address(&self) -> Address;
}

/// Bundler manager trait for submitting UserOperations
#[async_trait]
pub trait BundlerManager: Send + Sync {
    /// Send a UserOperation to the bundler
    async fn send_user_operation(
        &self,
        user_op: &UserOperation,
        entry_point: Address,
    ) -> Result<Hex>;

    /// Verify bundler supports the EntryPoint
    async fn verify_entry_point(&self, chain_id: ChainId, entry_point: Address) -> Result<()>;
}

#[async_trait]
impl MultiChainSmartAccount for BaseMultichainSmartAccount {
    fn address_on(&self, chain_id: ChainId) -> Result<Address> {
        self.addresses
            .get(&chain_id)
            .copied()
            .ok_or_else(|| crate::EilError::UnsupportedChain(chain_id))
    }

    async fn sign_user_ops(&self, mut user_ops: Vec<UserOperation>) -> Result<Vec<UserOperation>> {
        for user_op in &mut user_ops {
            let hash = compute_user_op_hash(user_op)?;
            let signature = self.signer.sign(&hash).await?;
            user_op.signature = signature;
        }
        Ok(user_ops)
    }

    async fn encode_calls(&self, _chain_id: ChainId, calls: Vec<Call>) -> Result<Hex> {
        // Simple batch encoding: just concatenate calldata
        // Real implementation would use account-specific encoding (e.g., ERC-4337 executeBatch)
        if calls.is_empty() {
            return Ok(Hex::new());
        }

        // This is a placeholder - actual encoding depends on the smart account implementation
        // For example, Safe uses a different encoding than Biconomy
        Ok(Hex::new())
    }

    async fn send_user_operation(&self, user_op: UserOperation) -> Result<Hex> {
        let entry_point = user_op
            .entry_point_address
            .ok_or_else(|| crate::EilError::Generic("EntryPoint address not set".into()))?;
        self.bundler_manager
            .send_user_operation(&user_op, entry_point)
            .await
    }

    async fn verify_bundler_config(
        &self,
        chain_id: ChainId,
        entry_point: Address,
    ) -> Result<()> {
        self.bundler_manager
            .verify_entry_point(chain_id, entry_point)
            .await
    }

    async fn get_nonce(&self, _chain_id: ChainId) -> Result<alloy::primitives::U256> {
        // Placeholder - would query EntryPoint contract
        Ok(alloy::primitives::U256::from(0))
    }

    async fn get_factory_args(&self, _chain_id: ChainId) -> Result<(Option<Address>, Option<Hex>)> {
        // Placeholder - would return factory and initCode if account not deployed
        Ok((None, None))
    }
}

/// Compute UserOperation hash for signing
fn compute_user_op_hash(user_op: &UserOperation) -> Result<[u8; 32]> {
    // Placeholder - actual implementation would use EIP-712 hashing
    // with proper domain separator including chainId and entryPoint
    Ok([0u8; 32])
}
