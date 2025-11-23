use crate::{config::CrossChainConfig, types::*, Result};
use std::collections::HashMap;

/// Simplified provider type - just store RPC URLs for now
/// Full provider implementation requires runtime async initialization
#[derive(Clone)]
pub struct NetworkEnvironment {
    /// RPC URLs per chain
    rpc_urls: HashMap<ChainId, String>,
    /// Configuration reference
    config: CrossChainConfig,
}

impl NetworkEnvironment {
    /// Create a new network environment from configuration
    pub fn new(config: &CrossChainConfig) -> Self {
        let mut rpc_urls = HashMap::new();

        for chain_info in &config.chain_infos {
            rpc_urls.insert(chain_info.chain_id, chain_info.rpc_url.clone());
        }

        Self {
            rpc_urls,
            config: config.clone(),
        }
    }

    /// Get RPC URL for a chain
    pub fn rpc_url(&self, chain_id: ChainId) -> Result<&str> {
        self.rpc_urls
            .get(&chain_id)
            .map(|s| s.as_str())
            .ok_or_else(|| crate::EilError::UnsupportedChain(chain_id))
    }

    /// Create a provider for a specific chain (async operation)
    /// Note: This is a placeholder. Full provider implementation requires proper async initialization
    pub async fn create_provider(
        &self,
        _chain_id: ChainId,
    ) -> Result<()> {
        // TODO: Implement proper provider creation
        // For now, this is a placeholder as provider creation in alloy
        // requires specific setup that varies by version
        Ok(())
    }

    /// Get all chain IDs
    pub fn chain_ids(&self) -> Vec<ChainId> {
        self.rpc_urls.keys().copied().collect()
    }

    /// Get configuration
    pub fn config(&self) -> &CrossChainConfig {
        &self.config
    }

    /// Get EntryPoint address for a chain
    pub fn entry_point(&self, chain_id: ChainId) -> Result<Address> {
        self.config
            .chain_info(chain_id)
            .map(|info| info.entry_point)
            .ok_or_else(|| crate::EilError::UnsupportedChain(chain_id))
    }

    /// Get Paymaster address for a chain
    pub fn paymaster(&self, chain_id: ChainId) -> Result<Address> {
        self.config
            .chain_info(chain_id)
            .map(|info| info.paymaster)
            .ok_or_else(|| crate::EilError::UnsupportedChain(chain_id))
    }
}
