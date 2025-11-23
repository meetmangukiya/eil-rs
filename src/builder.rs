use crate::{
    account::MultiChainSmartAccount,
    actions::Action,
    contract_types::*,
    network::NetworkEnvironment,
    types::*,
    voucher::{InternalVoucherInfo, VoucherCoordinator},
    Result,
};
use alloy::primitives::{keccak256, U256};
use std::{collections::HashSet, marker::PhantomData, sync::Arc};

/// Type-state for CrossChainBuilder
pub struct Building;
pub struct ReadyToBuild;
pub struct Signed;

/// Cross-chain builder for creating multi-chain operations
pub struct CrossChainBuilder<State = Building> {
    network_env: Arc<NetworkEnvironment>,
    batches: Vec<BatchBuilder>,
    coordinator: VoucherCoordinator,
    ephemeral_signer: Vec<u8>, // Simplified - would be proper key
    account: Option<Arc<dyn MultiChainSmartAccount>>,
    is_built: bool,
    _state: PhantomData<State>,
}

impl CrossChainBuilder<Building> {
    /// Create a new CrossChainBuilder
    pub fn new(network_env: &NetworkEnvironment) -> Self {
        // Generate ephemeral signer (simplified)
        let ephemeral_signer = vec![0u8; 32]; // Would use proper key generation

        Self {
            network_env: Arc::new(network_env.clone()),
            batches: Vec::new(),
            coordinator: VoucherCoordinator::new(),
            ephemeral_signer,
            account: None,
            is_built: false,
            _state: PhantomData,
        }
    }

    /// Set the account to use for this operation
    pub fn use_account(
        mut self,
        account: Arc<dyn MultiChainSmartAccount>,
    ) -> Result<CrossChainBuilder<ReadyToBuild>> {
        if self.account.is_some() {
            return Err(crate::EilError::AccountAlreadySet);
        }
        self.account = Some(account);

        Ok(CrossChainBuilder {
            network_env: self.network_env,
            batches: self.batches,
            coordinator: self.coordinator,
            ephemeral_signer: self.ephemeral_signer,
            account: self.account,
            is_built: self.is_built,
            _state: PhantomData,
        })
    }
}

impl CrossChainBuilder<ReadyToBuild> {
    /// Get the number of batches (for testing)
    /// Note: This is public for testing purposes only
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Start a new batch on the specified chain
    pub fn start_batch(mut self, chain_id: ChainId) -> BatchBuilder {
        let batch_index = self.batches.len();
        BatchBuilder::new(
            chain_id,
            batch_index,
            Arc::new(self.network_env.as_ref().clone()),
            self,
        )
    }

    /// Build all batches into SingleChainBatch objects
    pub async fn build_single_chain_batches(&mut self) -> Result<Vec<SingleChainBatch>> {
        self.assert_not_built()?;

        // Collect XLPs for each voucher
        self.collect_xlps_per_voucher().await?;

        // Build voucher requests
        self.build_vouchers().await?;

        // Validate all vouchers are consumed
        self.coordinator.validate_all_consumed()?;

        // Build each batch
        let mut batches = Vec::new();
        for batch_builder in &self.batches {
            let single_chain_batch = batch_builder.build_single_chain_batch().await?;
            batches.push(single_chain_batch);
        }

        Ok(batches)
    }

    /// Build and sign all UserOperations
    pub async fn build_and_sign(mut self) -> Result<crate::executor::CrossChainExecutor> {
        let batches = self.build_single_chain_batches().await?;

        let account = self
            .account
            .as_ref()
            .ok_or(crate::EilError::AccountNotSet)?;

        // Sign all UserOps
        let signed_user_ops = account.sign_user_ops(batches.iter().map(|b| b.user_op.clone()).collect()).await?;

        // Update batches with signatures
        let signed_batches: Vec<_> = batches
            .into_iter()
            .zip(signed_user_ops)
            .map(|(mut batch, signed_op)| {
                batch.user_op = signed_op;
                batch
            })
            .collect();

        self.is_built = true;

        Ok(crate::executor::CrossChainExecutor::new(
            Arc::new(self.network_env.as_ref().clone()),
            signed_batches,
        ))
    }

    fn assert_not_built(&self) -> Result<()> {
        if self.is_built {
            return Err(crate::EilError::BuilderAlreadyBuilt);
        }
        Ok(())
    }

    async fn collect_xlps_per_voucher(&mut self) -> Result<()> {
        // Collect all voucher ref_ids first to avoid borrow checker issues
        let ref_ids: Vec<String> = self
            .coordinator
            .all_vouchers()
            .iter()
            .map(|v| v.voucher.ref_id.clone())
            .collect();

        // For each voucher request, find solvent XLPs
        for ref_id in ref_ids {
            let voucher = self.coordinator.get(&ref_id)?.voucher.clone();
            let xlps = self.get_allowed_xlps(&voucher).await?;
            self.coordinator.set_allowed_xlps(&voucher.ref_id, xlps)?;
        }
        Ok(())
    }

    async fn get_allowed_xlps(&self, _voucher: &SdkVoucherRequest) -> Result<Vec<Address>> {
        // Placeholder - would query XLPs from paymaster contract
        // and filter based on XlpSelectionConfig
        Ok(vec![])
    }

    async fn build_vouchers(&mut self) -> Result<()> {
        // Collect all vouchers to avoid borrow checker issues
        let vouchers: Vec<SdkVoucherRequest> = self
            .coordinator
            .all_vouchers()
            .iter()
            .map(|v| v.voucher.clone())
            .collect();

        for voucher in vouchers {
            let voucher_request = self.sdk_to_voucher_request(&voucher).await?;
            self.coordinator
                .set_voucher_request(&voucher.ref_id, voucher_request)?;
        }
        Ok(())
    }

    async fn sdk_to_voucher_request(
        &self,
        sdk_request: &SdkVoucherRequest,
    ) -> Result<VoucherRequest> {
        let account = self
            .account
            .as_ref()
            .ok_or(crate::EilError::AccountNotSet)?;

        let source_chain = sdk_request.source_chain_id.unwrap();
        let dest_chain = sdk_request.destination_chain_id;

        let source_sender = account.address_on(source_chain)?;
        let dest_sender = sdk_request
            .target
            .unwrap_or_else(|| account.address_on(dest_chain).unwrap());

        // Get contract addresses
        let source_paymaster = self.network_env.paymaster(source_chain)?;
        let dest_paymaster = self.network_env.paymaster(dest_chain)?;

        // Convert tokens to assets
        let assets: Result<Vec<_>> = sdk_request
            .tokens
            .iter()
            .map(|t| {
                let token_addr = t
                    .token
                    .address_on(source_chain)
                    .ok_or_else(|| crate::EilError::InvalidAddress {
                        chain_id: source_chain,
                        address: format!("Token {} not deployed", t.token.name),
                    })?;
                let amount = match &t.amount {
                    Amount::Fixed(a) => *a,
                    Amount::Runtime(_) => {
                        t.min_provider_deposit
                            .unwrap_or(U256::from(1))
                    }
                };
                Ok(Asset {
                    erc20_token: token_addr,
                    amount,
                })
            })
            .collect();

        let source_assets = assets?;

        let dest_assets: Result<Vec<_>> = sdk_request
            .tokens
            .iter()
            .map(|t| {
                let token_addr = t
                    .token
                    .address_on(dest_chain)
                    .ok_or_else(|| crate::EilError::InvalidAddress {
                        chain_id: dest_chain,
                        address: format!("Token {} not deployed", t.token.name),
                    })?;
                let amount = match &t.amount {
                    Amount::Fixed(a) => *a,
                    Amount::Runtime(_) => {
                        t.min_provider_deposit
                            .unwrap_or(U256::from(1))
                    }
                };
                Ok(Asset {
                    erc20_token: token_addr,
                    amount,
                })
            })
            .collect();

        // Create fee rule from config
        let fee_config = &self.network_env.config().fee_config;
        let fee_rule = AtomicSwapFeeRule {
            start_fee_percent_numerator: crate::utils::fee_percent_to_numerator(
                fee_config.start_fee_percent,
            ),
            max_fee_percent_numerator: crate::utils::fee_percent_to_numerator(
                fee_config.max_fee_percent,
            ),
            fee_increase_per_second: crate::utils::fee_percent_to_numerator(
                fee_config.fee_increase_per_second,
            ),
            unspent_voucher_fee: crate::utils::fee_percent_to_numerator(
                fee_config.unspent_voucher_fee_percent,
            ),
        };

        // Get voucher info to access allowed XLPs
        let voucher_info = self.coordinator.get(&sdk_request.ref_id)?;
        let allowed_xlps = voucher_info.allowed_xlps.clone().unwrap_or_default();

        Ok(VoucherRequest {
            origination: SourceSwapComponent {
                chain_id: source_chain,
                sender: source_sender,
                paymaster: source_paymaster,
                assets: source_assets,
                fee_rule,
                sender_nonce: U256::from(0), // Would get actual nonce
                allowed_xlps,
            },
            destination: DestinationSwapComponent {
                chain_id: dest_chain,
                sender: dest_sender,
                paymaster: dest_paymaster,
                assets: dest_assets?,
                max_user_op_cost: U256::from(10_000_000_000_000_000u64), // 0.01 ETH
                expires_at: U256::from(
                    crate::utils::now_seconds() + self.network_env.config().expire_time_seconds,
                ),
            },
        })
    }

    /// Internal: add a completed batch
    fn add_batch(&mut self, batch: BatchBuilder) {
        self.batches.push(batch);
    }

    /// Internal: get coordinator
    fn coordinator_mut(&mut self) -> &mut VoucherCoordinator {
        &mut self.coordinator
    }
}

/// Batch builder for a single chain
pub struct BatchBuilder {
    chain_id: ChainId,
    batch_index: usize,
    actions: Vec<Box<dyn Action>>,
    input_vouchers: Vec<SdkVoucherRequest>,
    output_vouchers: Vec<SdkVoucherRequest>,
    vars: HashSet<String>,
    user_op_overrides: Option<UserOperation>,
    network_env: Arc<NetworkEnvironment>,
    parent_builder: Option<CrossChainBuilder<ReadyToBuild>>,
}

impl BatchBuilder {
    fn new(
        chain_id: ChainId,
        batch_index: usize,
        network_env: Arc<NetworkEnvironment>,
        parent: CrossChainBuilder<ReadyToBuild>,
    ) -> Self {
        Self {
            chain_id,
            batch_index,
            actions: Vec::new(),
            input_vouchers: Vec::new(),
            output_vouchers: Vec::new(),
            vars: HashSet::new(),
            user_op_overrides: None,
            network_env,
            parent_builder: Some(parent),
        }
    }

    /// Get the chain ID for this batch
    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    /// Add an action to this batch
    pub fn add_action(mut self, action: impl Action + 'static) -> Self {
        self.actions.push(Box::new(action));
        self
    }

    /// Add a voucher request
    pub fn add_voucher_request(mut self, mut request: SdkVoucherRequest) -> Self {
        request.source_chain_id = Some(self.chain_id);
        self.output_vouchers.push(request.clone());
        self
    }

    /// Use a voucher from another batch
    pub fn use_voucher(mut self, ref_id: impl Into<String>) -> Result<Self> {
        let ref_id = ref_id.into();
        let mut parent = self.parent_builder.take().unwrap();

        // Register voucher consumption
        parent
            .coordinator_mut()
            .mark_consumed(&ref_id, self.batch_index)?;

        let voucher_info = parent.coordinator.get(&ref_id)?;
        self.input_vouchers.push(voucher_info.voucher.clone());
        self.parent_builder = Some(parent);

        Ok(self)
    }

    /// End this batch and return to parent builder
    pub fn end_batch(mut self) -> CrossChainBuilder<ReadyToBuild> {
        let mut parent = self.parent_builder.take().unwrap();

        // Register output vouchers
        for voucher in &self.output_vouchers {
            parent
                .coordinator_mut()
                .register(voucher.clone(), self.batch_index)
                .expect("Failed to register voucher");
        }

        parent.add_batch(self);
        parent
    }

    /// Build this batch into a SingleChainBatch
    async fn build_single_chain_batch(&self) -> Result<SingleChainBatch> {
        let user_op = self.create_user_op().await?;
        let user_op_hash = compute_user_op_hash(&user_op)?;

        Ok(SingleChainBatch {
            user_op,
            user_op_hash,
            chain_id: self.chain_id,
            input_voucher_requests: self.input_vouchers.clone(),
            out_voucher_requests: self.output_vouchers.clone(),
        })
    }

    async fn create_user_op(&self) -> Result<UserOperation> {
        // Build calldata from actions
        let mut calls = Vec::new();
        for action in &self.actions {
            calls.extend(action.encode_call(self).await?);
        }

        // Encode calls
        // let call_data = if calls.is_empty() {
        //     Hex::new()
        // } else {
        //     // Would use account.encode_calls()
        //     Hex::new()
        // };

        // Create UserOperation
        Ok(UserOperation {
            sender: Address::ZERO, // Would get from account
            nonce: U256::from(0),
            factory: None,
            factory_data: None,
            call_data: Hex::new(),
            call_gas_limit: U256::from(3_000_000),
            verification_gas_limit: U256::from(500_000),
            pre_verification_gas: U256::from(100_000),
            max_fee_per_gas: U256::from(1_000_000_000), // 1 gwei
            max_priority_fee_per_gas: U256::from(1_000_000_000),
            paymaster: None,
            paymaster_verification_gas_limit: None,
            paymaster_post_op_gas_limit: None,
            paymaster_data: None,
            paymaster_signature: None,
            signature: Hex::new(),
            chain_id: Some(self.chain_id),
            entry_point_address: Some(self.network_env.entry_point(self.chain_id)?),
        })
    }
}

/// Compute UserOperation hash
fn compute_user_op_hash(_user_op: &UserOperation) -> Result<Hex> {
    // Simplified - would use proper EIP-712 encoding
    let hash = keccak256(&[0u8; 32]);
    Ok(Hex::from(hash.to_vec()))
}
