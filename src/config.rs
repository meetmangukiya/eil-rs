use crate::types::*;
use serde::{Deserialize, Serialize};

/// Chain information configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    /// Chain ID
    pub chain_id: ChainId,
    /// RPC URL for this chain
    pub rpc_url: String,
    /// EntryPoint contract address
    pub entry_point: Address,
    /// CrossChainPaymaster contract address
    pub paymaster: Address,
    /// Optional bundler URL (if different from RPC)
    pub bundler_url: Option<String>,
}

/// XLP (Cross-chain Liquidity Provider) selection configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct XlpSelectionConfig {
    /// Reserve factor for deposits (e.g., 1.1 means require 110% of requested amount)
    #[serde(default = "default_deposit_reserve_factor")]
    pub deposit_reserve_factor: f64,

    /// Whether to include balance checks
    #[serde(default = "default_include_balance")]
    pub include_balance: bool,

    /// Minimum number of XLPs required
    #[serde(default = "default_min_xlps")]
    pub min_xlps: usize,

    /// Maximum number of XLPs to use
    #[serde(default = "default_max_xlps")]
    pub max_xlps: usize,

    /// Custom XLP filter function (not serializable, must be set programmatically)
    #[serde(skip)]
    pub custom_xlp_filter: Option<XlpFilterFn>,
}

impl Default for XlpSelectionConfig {
    fn default() -> Self {
        Self {
            deposit_reserve_factor: default_deposit_reserve_factor(),
            include_balance: default_include_balance(),
            min_xlps: default_min_xlps(),
            max_xlps: default_max_xlps(),
            custom_xlp_filter: None,
        }
    }
}

/// Custom XLP filter function type
pub type XlpFilterFn = std::sync::Arc<
    dyn Fn(ChainId, Address, Address, alloy::primitives::U256, alloy::primitives::U256) -> bool
        + Send
        + Sync,
>;

fn default_deposit_reserve_factor() -> f64 {
    1.0
}

fn default_include_balance() -> bool {
    false
}

fn default_min_xlps() -> usize {
    1
}

fn default_max_xlps() -> usize {
    5
}

/// Fee configuration for vouchers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeConfig {
    /// Starting fee percentage (0.0 to 1.0, e.g., 0.001 = 0.1%)
    #[serde(default = "default_start_fee_percent")]
    pub start_fee_percent: f64,

    /// Maximum fee percentage (0.0 to 1.0)
    #[serde(default = "default_max_fee_percent")]
    pub max_fee_percent: f64,

    /// Fee increase per second (0.0 to 1.0)
    #[serde(default = "default_fee_increase_per_second")]
    pub fee_increase_per_second: f64,

    /// Unspent voucher fee percentage (0.0 to 1.0)
    #[serde(default = "default_unspent_voucher_fee_percent")]
    pub unspent_voucher_fee_percent: f64,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            start_fee_percent: default_start_fee_percent(),
            max_fee_percent: default_max_fee_percent(),
            fee_increase_per_second: default_fee_increase_per_second(),
            unspent_voucher_fee_percent: default_unspent_voucher_fee_percent(),
        }
    }
}

fn default_start_fee_percent() -> f64 {
    0.001
}

fn default_max_fee_percent() -> f64 {
    0.05
}

fn default_fee_increase_per_second() -> f64 {
    0.0001
}

fn default_unspent_voucher_fee_percent() -> f64 {
    0.001
}

/// Source chain paymaster interface (for chains without vouchers)
pub trait SourcePaymaster: Send + Sync {
    /// Get paymaster stub data for UserOp
    fn get_paymaster_stub_data(
        &self,
        user_op: &crate::contract_types::UserOperation,
    ) -> crate::Result<PaymasterData>;
}

/// Paymaster data
#[derive(Debug, Clone)]
pub struct PaymasterData {
    /// Paymaster address
    pub paymaster: Option<Address>,
    /// Paymaster data
    pub paymaster_data: Option<Hex>,
    /// Paymaster verification gas limit
    pub paymaster_verification_gas_limit: Option<alloy::primitives::U256>,
    /// Paymaster post-op gas limit
    pub paymaster_post_op_gas_limit: Option<alloy::primitives::U256>,
}

/// Main cross-chain configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct CrossChainConfig {
    /// Voucher expiration time in seconds
    #[serde(default = "default_expire_time_seconds")]
    pub expire_time_seconds: u64,

    /// Execution timeout in seconds
    #[serde(default = "default_exec_timeout_seconds")]
    pub exec_timeout_seconds: u64,

    /// XLP selection configuration
    #[serde(default)]
    pub xlp_selection_config: XlpSelectionConfig,

    /// Fee configuration
    #[serde(default)]
    pub fee_config: FeeConfig,

    /// Per-chain configuration
    pub chain_infos: Vec<ChainInfo>,

    /// Source paymaster (not serializable, must be set programmatically)
    #[serde(skip)]
    pub source_paymaster: Option<std::sync::Arc<dyn SourcePaymaster>>,
}

impl Default for CrossChainConfig {
    fn default() -> Self {
        Self {
            expire_time_seconds: default_expire_time_seconds(),
            exec_timeout_seconds: default_exec_timeout_seconds(),
            xlp_selection_config: XlpSelectionConfig::default(),
            fee_config: FeeConfig::default(),
            chain_infos: Vec::new(),
            source_paymaster: None,
        }
    }
}

fn default_expire_time_seconds() -> u64 {
    60
}

fn default_exec_timeout_seconds() -> u64 {
    30
}

impl CrossChainConfig {
    /// Create a new configuration with the given chain infos
    pub fn new(chain_infos: Vec<ChainInfo>) -> Self {
        Self {
            chain_infos,
            ..Default::default()
        }
    }

    /// Add a chain configuration
    pub fn add_chain(mut self, chain_info: ChainInfo) -> Self {
        self.chain_infos.push(chain_info);
        self
    }

    /// Set XLP selection configuration
    pub fn with_xlp_config(mut self, config: XlpSelectionConfig) -> Self {
        self.xlp_selection_config = config;
        self
    }

    /// Set fee configuration
    pub fn with_fee_config(mut self, config: FeeConfig) -> Self {
        self.fee_config = config;
        self
    }

    /// Set expiration time
    pub fn with_expire_time(mut self, seconds: u64) -> Self {
        self.expire_time_seconds = seconds;
        self
    }

    /// Set execution timeout
    pub fn with_exec_timeout(mut self, seconds: u64) -> Self {
        self.exec_timeout_seconds = seconds;
        self
    }

    /// Set source paymaster
    pub fn with_source_paymaster(
        mut self,
        paymaster: std::sync::Arc<dyn SourcePaymaster>,
    ) -> Self {
        self.source_paymaster = Some(paymaster);
        self
    }

    /// Get chain info for a specific chain
    pub fn chain_info(&self, chain_id: ChainId) -> Option<&ChainInfo> {
        self.chain_infos.iter().find(|c| c.chain_id == chain_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_chain_info(chain_id: u64) -> ChainInfo {
        ChainInfo {
            chain_id,
            rpc_url: format!("https://rpc-{}.example.com", chain_id),
            entry_point: "0x0000000071727De22E5E9d8BAf0edAc6f37da032".parse().unwrap(),
            paymaster: "0x0000000000000000000000000000000000000001".parse().unwrap(),
            bundler_url: None,
        }
    }

    #[test]
    fn test_xlp_selection_config_defaults() {
        let config = XlpSelectionConfig::default();
        assert_eq!(config.deposit_reserve_factor, 1.0);
        assert_eq!(config.include_balance, false);
        assert_eq!(config.min_xlps, 1);
        assert_eq!(config.max_xlps, 5);
    }

    #[test]
    fn test_fee_config_defaults() {
        let config = FeeConfig::default();
        assert_eq!(config.start_fee_percent, 0.001);
        assert_eq!(config.max_fee_percent, 0.05);
        assert_eq!(config.fee_increase_per_second, 0.0001);
        assert_eq!(config.unspent_voucher_fee_percent, 0.001);
    }

    #[test]
    fn test_cross_chain_config_defaults() {
        let config = CrossChainConfig::default();
        assert_eq!(config.expire_time_seconds, 60);
        assert_eq!(config.exec_timeout_seconds, 30);
        assert_eq!(config.chain_infos.len(), 0);
    }

    #[test]
    fn test_cross_chain_config_builder() {
        let config = CrossChainConfig::new(vec![create_test_chain_info(1)])
            .with_expire_time(120)
            .with_exec_timeout(60);

        assert_eq!(config.expire_time_seconds, 120);
        assert_eq!(config.exec_timeout_seconds, 60);
        assert_eq!(config.chain_infos.len(), 1);
    }

    #[test]
    fn test_cross_chain_config_add_chain() {
        let config = CrossChainConfig::default()
            .add_chain(create_test_chain_info(1))
            .add_chain(create_test_chain_info(2));

        assert_eq!(config.chain_infos.len(), 2);
    }

    #[test]
    fn test_cross_chain_config_chain_info() {
        let chain1 = create_test_chain_info(1);
        let chain2 = create_test_chain_info(2);

        let config = CrossChainConfig::default()
            .add_chain(chain1)
            .add_chain(chain2);

        assert!(config.chain_info(1).is_some());
        assert!(config.chain_info(2).is_some());
        assert!(config.chain_info(3).is_none());
    }

    #[test]
    fn test_cross_chain_config_with_xlp_config() {
        let xlp_config = XlpSelectionConfig {
            deposit_reserve_factor: 1.5,
            include_balance: true,
            min_xlps: 2,
            max_xlps: 10,
            custom_xlp_filter: None,
        };

        let config = CrossChainConfig::default()
            .with_xlp_config(xlp_config.clone());

        assert_eq!(config.xlp_selection_config.deposit_reserve_factor, 1.5);
        assert_eq!(config.xlp_selection_config.include_balance, true);
        assert_eq!(config.xlp_selection_config.min_xlps, 2);
        assert_eq!(config.xlp_selection_config.max_xlps, 10);
    }

    #[test]
    fn test_cross_chain_config_with_fee_config() {
        let fee_config = FeeConfig {
            start_fee_percent: 0.002,
            max_fee_percent: 0.1,
            fee_increase_per_second: 0.0002,
            unspent_voucher_fee_percent: 0.002,
        };

        let config = CrossChainConfig::default()
            .with_fee_config(fee_config.clone());

        assert_eq!(config.fee_config.start_fee_percent, 0.002);
        assert_eq!(config.fee_config.max_fee_percent, 0.1);
    }
}
