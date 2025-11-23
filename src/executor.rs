use crate::{
    contract_types::{BatchStatusInfo, SdkVoucherRequest, SingleChainBatch},
    network::NetworkEnvironment,
    types::*,
    Result,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Callback type for execution events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackType {
    /// Submitting a UserOperation for execution
    Executing,
    /// UserOperation completed execution
    Done,
    /// UserOperation execution reverted
    Failed,
    /// Waiting for vouchers to be signed
    WaitingForVouchers,
    /// A voucher was signed by a provider
    VoucherIssued,
}

/// Execution callback data
#[derive(Debug, Clone)]
pub struct ExecCallbackData {
    /// Batch index
    pub index: usize,
    /// Callback type
    pub callback_type: CallbackType,
    /// UserOperation being executed
    pub user_op_hash: Hex,
    /// Transaction hash (if executed)
    pub tx_hash: Option<Hex>,
    /// Request IDs (for vouchers)
    pub request_ids: Option<Vec<Hex>>,
    /// Revert reason (if failed)
    pub revert_reason: Option<String>,
    /// Input voucher requests
    pub input_voucher_requests: Vec<SdkVoucherRequest>,
    /// Output voucher requests
    pub out_voucher_requests: Vec<SdkVoucherRequest>,
}

/// Execution callback function type
pub type ExecCallback = Box<dyn Fn(ExecCallbackData) + Send + Sync>;

/// Cross-chain executor
/// Executes signed UserOperations across multiple chains
pub struct CrossChainExecutor {
    network_env: Arc<NetworkEnvironment>,
    batches: Vec<SingleChainBatch>,
    timeout_seconds: u64,
}

impl CrossChainExecutor {
    /// Create a new executor
    pub fn new(network_env: Arc<NetworkEnvironment>, batches: Vec<SingleChainBatch>) -> Self {
        let timeout_seconds = network_env.config().exec_timeout_seconds;
        Self {
            network_env,
            batches,
            timeout_seconds,
        }
    }

    /// Execute all batches
    pub async fn execute<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(ExecCallbackData) + Send + Sync,
    {
        // Validate all UserOps are signed
        for batch in &self.batches {
            if batch.user_op.signature.is_empty() {
                return Err(crate::EilError::UserOpNotSigned);
            }
        }

        // Initialize batch status
        let mut batch_statuses: Vec<BatchStatusInfo> = self
            .batches
            .iter()
            .enumerate()
            .map(|(index, batch)| BatchStatusInfo {
                index,
                batch: batch.clone(),
                status: OperationStatus::Pending,
                vouchers: std::collections::HashMap::new(),
                request_ids: None,
                tx_hash: None,
                revert_reason: None,
            })
            .collect();

        // Watch for voucher events (would start event listeners in real impl)
        // self.watch_for_voucher_events(&batch_statuses, &callback).await;

        // Execution loop
        let start_time = std::time::Instant::now();
        loop {
            // Check timeout
            if start_time.elapsed().as_secs() > self.timeout_seconds {
                return Err(crate::EilError::ExecutionTimeout(self.timeout_seconds));
            }

            // Check if all done
            if batch_statuses
                .iter()
                .all(|b| b.status == OperationStatus::Done || b.status == OperationStatus::Failed)
            {
                break;
            }

            // Find batch ready to execute
            if let Some(batch_info) = self.find_ready_batch(&batch_statuses).await? {
                self.execute_single_batch(batch_info, &callback).await?;
            } else {
                // Wait for events
                sleep(Duration::from_millis(100)).await;
            }
        }

        Ok(())
    }

    /// Find a batch that's ready to execute
    async fn find_ready_batch<'a>(
        &self,
        batches: &'a [BatchStatusInfo],
    ) -> Result<Option<&'a BatchStatusInfo>> {
        for batch in batches {
            if batch.status != OperationStatus::Pending {
                continue;
            }

            // Check if waiting for vouchers
            if self.is_waiting_for_vouchers(batch).await? {
                continue;
            }

            return Ok(Some(batch));
        }
        Ok(None)
    }

    /// Check if batch is waiting for vouchers
    async fn is_waiting_for_vouchers(&self, batch: &BatchStatusInfo) -> Result<bool> {
        for _voucher_req in &batch.batch.input_voucher_requests {
            // Would check if voucher is signed
            // For now, assume all vouchers are ready
        }
        Ok(false)
    }

    /// Execute a single batch
    async fn execute_single_batch<F>(&self, batch: &BatchStatusInfo, callback: &F) -> Result<()>
    where
        F: Fn(ExecCallbackData) + Send + Sync,
    {
        // Call callback: Executing
        callback(ExecCallbackData {
            index: batch.index,
            callback_type: CallbackType::Executing,
            user_op_hash: batch.batch.user_op_hash.clone(),
            tx_hash: None,
            request_ids: None,
            revert_reason: None,
            input_voucher_requests: batch.batch.input_voucher_requests.clone(),
            out_voucher_requests: batch.batch.out_voucher_requests.clone(),
        });

        // Submit UserOp to bundler
        // In real implementation, would:
        // 1. Send UserOp to bundler
        // 2. Wait for inclusion
        // 3. Watch for events

        // Placeholder: assume success
        callback(ExecCallbackData {
            index: batch.index,
            callback_type: CallbackType::Done,
            user_op_hash: batch.batch.user_op_hash.clone(),
            tx_hash: Some(Hex::new()), // Would be actual tx hash
            request_ids: None,
            revert_reason: None,
            input_voucher_requests: batch.batch.input_voucher_requests.clone(),
            out_voucher_requests: batch.batch.out_voucher_requests.clone(),
        });

        Ok(())
    }
}
