//! # EIL SDK - Ethereum Interoperability Layer SDK for Rust
//!
//! This SDK provides the necessary tools to integrate EIL into wallets and dApps,
//! allowing users to execute complex multi-chain operations as if they were performing
//! a single transaction on one unified network.
//!
//! ## Features
//!
//! - **Trustless Interoperability**: No third-party trust required
//! - **Single-Signature Cross-Chain Operations**: Sign once, execute across multiple chains
//! - **Cross-Chain Gas Payment**: Pay gas on any chain using XLPs
//! - **Multi-Chain Execution**: Seamless batch operations across L2s
//!
//! ## Example
//!
//! ```rust,ignore
//! use eil::{EilSdk, actions::*};
//!
//! let sdk = EilSdk::new(config);
//! let usdc = sdk.create_token("USDC", token_deployments);
//!
//! let executor = sdk.create_builder()
//!     .use_account(account)
//!     .start_batch(ChainId::Optimism)
//!         .add_voucher_request(VoucherRequest { /* ... */ })
//!         .end_batch()
//!     .start_batch(ChainId::Arbitrum)
//!         .use_voucher("voucher1")
//!         .add_action(ApproveAction { /* ... */ })
//!         .end_batch()
//!     .build_and_sign().await?;
//!
//! executor.execute(|event| {
//!     println!("Event: {:?}", event);
//! }).await?;
//! ```

pub mod types;
pub mod config;
pub mod contract_types;
pub mod multichain;
pub mod actions;
pub mod voucher;
pub mod builder;
pub mod executor;
pub mod account;
pub mod network;
pub mod utils;

mod error;

// Test utilities - exposed for integration tests
// Note: Only use in test code
pub mod test_utils;

pub use error::{EilError, Result};
pub use types::*;

/// Main entry point for the EIL SDK
pub struct EilSdk {
    config: config::CrossChainConfig,
    network_env: network::NetworkEnvironment,
}

impl EilSdk {
    /// Create a new EIL SDK instance with the given configuration
    pub fn new(config: config::CrossChainConfig) -> Self {
        let network_env = network::NetworkEnvironment::new(&config);
        Self { config, network_env }
    }

    /// Create a new CrossChainBuilder for building multi-chain operations
    pub fn create_builder(&self) -> builder::CrossChainBuilder {
        builder::CrossChainBuilder::new(&self.network_env)
    }

    /// Create a MultichainToken with the given deployment addresses
    pub fn create_token(
        &self,
        name: impl Into<String>,
        deployments: multichain::AddressPerChain,
    ) -> multichain::MultichainToken {
        multichain::MultichainToken::new(name.into(), deployments)
    }

    /// Get the network environment
    pub fn network_env(&self) -> &network::NetworkEnvironment {
        &self.network_env
    }
}
