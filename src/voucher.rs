use crate::{
    contract_types::{SdkVoucherRequest, Voucher, VoucherRequest},
    types::*,
    Result,
};
use std::collections::HashMap;

/// Internal voucher information tracking
#[derive(Debug, Clone)]
pub struct InternalVoucherInfo {
    /// The SDK voucher request
    pub voucher: SdkVoucherRequest,
    /// Source batch index
    pub source_batch_index: usize,
    /// Destination batch index (once consumed)
    pub dest_batch_index: Option<usize>,
    /// Converted contract voucher request
    pub voucher_request: Option<VoucherRequest>,
    /// Allowed XLPs for this voucher
    pub allowed_xlps: Option<Vec<Address>>,
    /// Signed voucher (once received from XLP)
    pub signed_voucher: Option<Voucher>,
}

/// Coordinates voucher requests across batches
#[derive(Debug, Clone, Default)]
pub struct VoucherCoordinator {
    /// Map of voucher ref ID to internal info
    vouchers: HashMap<String, InternalVoucherInfo>,
}

impl VoucherCoordinator {
    /// Create a new voucher coordinator
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new voucher request
    pub fn register(
        &mut self,
        voucher: SdkVoucherRequest,
        source_batch_index: usize,
    ) -> Result<()> {
        if self.vouchers.contains_key(&voucher.ref_id) {
            return Err(crate::EilError::DuplicateVoucher(voucher.ref_id.clone()));
        }

        self.vouchers.insert(
            voucher.ref_id.clone(),
            InternalVoucherInfo {
                voucher,
                source_batch_index,
                dest_batch_index: None,
                voucher_request: None,
                allowed_xlps: None,
                signed_voucher: None,
            },
        );

        Ok(())
    }

    /// Mark a voucher as consumed by a batch
    pub fn mark_consumed(&mut self, ref_id: &str, dest_batch_index: usize) -> Result<()> {
        let info = self
            .vouchers
            .get_mut(ref_id)
            .ok_or_else(|| crate::EilError::VoucherNotFound(ref_id.to_string()))?;

        if info.dest_batch_index.is_some() {
            return Err(crate::EilError::VoucherAlreadyUsed(ref_id.to_string()));
        }

        info.dest_batch_index = Some(dest_batch_index);
        Ok(())
    }

    /// Get voucher info
    pub fn get(&self, ref_id: &str) -> Result<&InternalVoucherInfo> {
        self.vouchers
            .get(ref_id)
            .ok_or_else(|| crate::EilError::VoucherNotFound(ref_id.to_string()))
    }

    /// Get mutable voucher info
    pub fn get_mut(&mut self, ref_id: &str) -> Result<&mut InternalVoucherInfo> {
        self.vouchers
            .get_mut(ref_id)
            .ok_or_else(|| crate::EilError::VoucherNotFound(ref_id.to_string()))
    }

    /// Update voucher with contract VoucherRequest
    pub fn set_voucher_request(
        &mut self,
        ref_id: &str,
        voucher_request: VoucherRequest,
    ) -> Result<()> {
        let info = self.get_mut(ref_id)?;
        info.voucher_request = Some(voucher_request);
        Ok(())
    }

    /// Update allowed XLPs for a voucher
    pub fn set_allowed_xlps(&mut self, ref_id: &str, xlps: Vec<Address>) -> Result<()> {
        let info = self.get_mut(ref_id)?;
        info.allowed_xlps = Some(xlps);
        Ok(())
    }

    /// Set signed voucher
    pub fn set_signed_voucher(&mut self, ref_id: &str, voucher: Voucher) -> Result<()> {
        let info = self.get_mut(ref_id)?;
        info.signed_voucher = Some(voucher);
        Ok(())
    }

    /// Get all voucher requests
    pub fn all_vouchers(&self) -> Vec<&InternalVoucherInfo> {
        self.vouchers.values().collect()
    }

    /// Get all unconsumed vouchers
    pub fn unconsumed_vouchers(&self) -> Vec<&InternalVoucherInfo> {
        self.vouchers
            .values()
            .filter(|v| v.dest_batch_index.is_none())
            .collect()
    }

    /// Validate all vouchers are consumed
    pub fn validate_all_consumed(&self) -> Result<()> {
        for (ref_id, info) in &self.vouchers {
            if info.dest_batch_index.is_none() {
                return Err(crate::EilError::VoucherNotConsumed(
                    ref_id.clone(),
                    info.voucher.source_chain_id.unwrap_or(0),
                ));
            }
        }
        Ok(())
    }
}

/// XLP (Cross-chain Liquidity Provider) information with solvency data
#[derive(Debug, Clone)]
pub struct SolventXlpInfo {
    /// XLP entry
    pub xlp_entry: crate::contract_types::XlpEntry,
    /// Deposits per token
    pub deposits: Vec<alloy::primitives::U256>,
    /// Balances per token
    pub balances: Vec<alloy::primitives::U256>,
}

/// Get solvent XLPs for a destination chain
/// This is a placeholder - actual implementation would query the paymaster contract
pub async fn get_solvent_xlps(
    _chain_id: ChainId,
    _paymaster: Address,
    _tokens: &[TokenAmount],
    _include_balance: bool,
) -> Result<Vec<SolventXlpInfo>> {
    // TODO: Implement actual XLP querying logic
    // This would involve:
    // 1. Querying the paymaster contract for all XLPs
    // 2. Checking their deposits and balances
    // 3. Filtering for solvent ones
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multichain::MultichainToken;
    use std::collections::HashMap;

    fn create_test_voucher(ref_id: &str, dest_chain: u64) -> SdkVoucherRequest {
        let mut deployments = HashMap::new();
        deployments.insert(1, "0x0000000000000000000000000000000000000001".parse().unwrap());
        deployments.insert(dest_chain, "0x0000000000000000000000000000000000000010".parse().unwrap());

        let token = MultichainToken::new("TEST".to_string(), deployments);

        SdkVoucherRequest {
            ref_id: ref_id.to_string(),
            source_chain_id: Some(1),
            destination_chain_id: dest_chain,
            tokens: vec![TokenAmount {
                token,
                amount: Amount::Fixed(alloy::primitives::U256::from(100)),
                min_provider_deposit: None,
            }],
            target: None,
        }
    }

    #[test]
    fn test_voucher_coordinator_new() {
        let coordinator = VoucherCoordinator::new();
        assert_eq!(coordinator.all_vouchers().len(), 0);
    }

    #[test]
    fn test_voucher_coordinator_register() {
        let mut coordinator = VoucherCoordinator::new();
        let voucher = create_test_voucher("v1", 10);

        let result = coordinator.register(voucher, 0);
        assert!(result.is_ok());
        assert_eq!(coordinator.all_vouchers().len(), 1);
    }

    #[test]
    fn test_voucher_coordinator_register_duplicate() {
        let mut coordinator = VoucherCoordinator::new();
        let voucher1 = create_test_voucher("v1", 10);
        let voucher2 = create_test_voucher("v1", 10);

        coordinator.register(voucher1, 0).unwrap();
        let result = coordinator.register(voucher2, 1);

        assert!(result.is_err());
        match result {
            Err(crate::EilError::DuplicateVoucher(ref_id)) => {
                assert_eq!(ref_id, "v1");
            }
            _ => panic!("Expected DuplicateVoucher error"),
        }
    }

    #[test]
    fn test_voucher_coordinator_get() {
        let mut coordinator = VoucherCoordinator::new();
        let voucher = create_test_voucher("v1", 10);

        coordinator.register(voucher, 0).unwrap();

        let result = coordinator.get("v1");
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.voucher.ref_id, "v1");
        assert_eq!(info.source_batch_index, 0);
        assert!(info.dest_batch_index.is_none());
    }

    #[test]
    fn test_voucher_coordinator_get_not_found() {
        let coordinator = VoucherCoordinator::new();
        let result = coordinator.get("nonexistent");

        assert!(result.is_err());
        match result {
            Err(crate::EilError::VoucherNotFound(ref_id)) => {
                assert_eq!(ref_id, "nonexistent");
            }
            _ => panic!("Expected VoucherNotFound error"),
        }
    }

    #[test]
    fn test_voucher_coordinator_mark_consumed() {
        let mut coordinator = VoucherCoordinator::new();
        let voucher = create_test_voucher("v1", 10);

        coordinator.register(voucher, 0).unwrap();
        let result = coordinator.mark_consumed("v1", 1);

        assert!(result.is_ok());

        let info = coordinator.get("v1").unwrap();
        assert_eq!(info.dest_batch_index, Some(1));
    }

    #[test]
    fn test_voucher_coordinator_mark_consumed_twice() {
        let mut coordinator = VoucherCoordinator::new();
        let voucher = create_test_voucher("v1", 10);

        coordinator.register(voucher, 0).unwrap();
        coordinator.mark_consumed("v1", 1).unwrap();

        let result = coordinator.mark_consumed("v1", 2);
        assert!(result.is_err());
        match result {
            Err(crate::EilError::VoucherAlreadyUsed(ref_id)) => {
                assert_eq!(ref_id, "v1");
            }
            _ => panic!("Expected VoucherAlreadyUsed error"),
        }
    }

    #[test]
    fn test_voucher_coordinator_unconsumed_vouchers() {
        let mut coordinator = VoucherCoordinator::new();

        coordinator.register(create_test_voucher("v1", 10), 0).unwrap();
        coordinator.register(create_test_voucher("v2", 10), 0).unwrap();
        coordinator.register(create_test_voucher("v3", 10), 0).unwrap();

        coordinator.mark_consumed("v1", 1).unwrap();

        let unconsumed = coordinator.unconsumed_vouchers();
        assert_eq!(unconsumed.len(), 2);
    }

    #[test]
    fn test_voucher_coordinator_validate_all_consumed_success() {
        let mut coordinator = VoucherCoordinator::new();

        coordinator.register(create_test_voucher("v1", 10), 0).unwrap();
        coordinator.register(create_test_voucher("v2", 10), 0).unwrap();

        coordinator.mark_consumed("v1", 1).unwrap();
        coordinator.mark_consumed("v2", 2).unwrap();

        let result = coordinator.validate_all_consumed();
        assert!(result.is_ok());
    }

    #[test]
    fn test_voucher_coordinator_validate_all_consumed_failure() {
        let mut coordinator = VoucherCoordinator::new();

        coordinator.register(create_test_voucher("v1", 10), 0).unwrap();
        coordinator.register(create_test_voucher("v2", 10), 0).unwrap();

        coordinator.mark_consumed("v1", 1).unwrap();
        // v2 not consumed

        let result = coordinator.validate_all_consumed();
        assert!(result.is_err());
        match result {
            Err(crate::EilError::VoucherNotConsumed(ref_id, _)) => {
                assert_eq!(ref_id, "v2");
            }
            _ => panic!("Expected VoucherNotConsumed error"),
        }
    }

    #[test]
    fn test_voucher_coordinator_set_allowed_xlps() {
        let mut coordinator = VoucherCoordinator::new();
        let voucher = create_test_voucher("v1", 10);

        coordinator.register(voucher, 0).unwrap();

        let xlps = vec![
            "0x0000000000000000000000000000000000000001".parse().unwrap(),
            "0x0000000000000000000000000000000000000002".parse().unwrap(),
        ];

        let result = coordinator.set_allowed_xlps("v1", xlps.clone());
        assert!(result.is_ok());

        let info = coordinator.get("v1").unwrap();
        assert_eq!(info.allowed_xlps.as_ref().unwrap().len(), 2);
    }
}
