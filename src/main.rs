use eil::{
    config::*,
    multichain::AddressPerChain,
    types::*,
    EilSdk,
};
use alloy::primitives::U256;
use std::collections::HashMap;

/// Example: Alice uses 90 USDC on Optimism to buy an NFT on Arbitrum
#[tokio::main]
async fn main() -> eil::Result<()> {
    println!("EIL SDK Example - Cross-chain NFT purchase");
    println!("===========================================\n");

    // 1. Setup configuration
    let config = setup_config();

    // 2. Create SDK instance
    let sdk = EilSdk::new(config);

    // 3. Create USDC token (deployed on multiple chains)
    let usdc = sdk.create_token("USDC", create_usdc_deployments());

    println!("✓ SDK initialized");
    println!("✓ USDC token configured on {} chains\n", usdc.deployments.len());

    // 4. Create account (placeholder - would use real account implementation)
    // let account = create_account();

    // 5. Build cross-chain operation
    println!("Building cross-chain operation...");

    /* Example usage (commented out as we need a real account implementation):

    let executor = sdk
        .create_builder()
        .use_account(account)?
        // Batch 1: On Optimism - create voucher to send 90 USDC to Arbitrum
        .start_batch(chain_ids::OPTIMISM)
            .add_voucher_request(SdkVoucherRequest {
                ref_id: "voucher1".to_string(),
                source_chain_id: None, // Will be set to current batch chain
                destination_chain_id: chain_ids::ARBITRUM,
                tokens: vec![TokenAmount {
                    token: usdc.clone(),
                    amount: Amount::Fixed(U256::from(90_000000)), // 90 USDC (6 decimals)
                    min_provider_deposit: None,
                }],
                target: None, // Use same address on destination
            })
            .end_batch()
        // Batch 2: On Arbitrum - use voucher and purchase NFT
        .start_batch(chain_ids::ARBITRUM)
            .use_voucher("voucher1")?
            .add_action(ApproveAction {
                token: usdc.clone(),
                spender: nft_marketplace_address(),
                value: Amount::Fixed(U256::from(90_000000)),
            })
            .add_action(FunctionCallAction {
                call: create_purchase_nft_call(123),
            })
            .end_batch()
        .build_and_sign()
        .await?;

    println!("✓ Operation built and signed\n");

    // 6. Execute across chains
    println!("Executing cross-chain operation...");

    executor
        .execute(|event| {
            match event.callback_type {
                eil::executor::CallbackType::Executing => {
                    println!("  → Executing batch {} on chain...", event.index);
                }
                eil::executor::CallbackType::Done => {
                    println!("  ✓ Batch {} completed", event.index);
                    if let Some(tx_hash) = event.tx_hash {
                        println!("    Tx: {}", hex::encode(tx_hash));
                    }
                }
                eil::executor::CallbackType::Failed => {
                    println!("  ✗ Batch {} failed", event.index);
                    if let Some(reason) = event.revert_reason {
                        println!("    Reason: {}", reason);
                    }
                }
                eil::executor::CallbackType::WaitingForVouchers => {
                    println!("  ⏳ Batch {} waiting for vouchers...", event.index);
                }
                eil::executor::CallbackType::VoucherIssued => {
                    println!("  ✓ Voucher issued for batch {}", event.index);
                }
            }
        })
        .await?;

    println!("\n✓ Cross-chain operation completed successfully!");
    */

    println!("\n📝 Note: This is a demonstration of the API structure.");
    println!("   A full implementation requires:");
    println!("   - MultiChainSmartAccount implementation (e.g., Safe, Biconomy)");
    println!("   - Bundler integration for submitting UserOps");
    println!("   - Event polling for voucher signatures");
    println!("   - Contract ABI definitions");

    Ok(())
}

/// Setup cross-chain configuration
fn setup_config() -> CrossChainConfig {
    CrossChainConfig::new(vec![
        ChainInfo {
            chain_id: chain_ids::OPTIMISM,
            rpc_url: "https://optimism.llamarpc.com".to_string(),
            entry_point: "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
                .parse()
                .unwrap(),
            paymaster: "0x0000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
            bundler_url: None,
        },
        ChainInfo {
            chain_id: chain_ids::ARBITRUM,
            rpc_url: "https://arbitrum.llamarpc.com".to_string(),
            entry_point: "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
                .parse()
                .unwrap(),
            paymaster: "0x0000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
            bundler_url: None,
        },
    ])
    .with_expire_time(60)
    .with_exec_timeout(30)
}

/// Create USDC token deployments across chains
fn create_usdc_deployments() -> AddressPerChain {
    let mut deployments = HashMap::new();

    // USDC on Optimism
    deployments.insert(
        chain_ids::OPTIMISM,
        "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85"
            .parse()
            .unwrap(),
    );

    // USDC on Arbitrum
    deployments.insert(
        chain_ids::ARBITRUM,
        "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"
            .parse()
            .unwrap(),
    );

    deployments
}

/// NFT marketplace address (example)
fn nft_marketplace_address() -> Address {
    "0x0000000000000000000000000000000000000001"
        .parse()
        .unwrap()
}

/// Create purchase NFT function call (example)
fn create_purchase_nft_call(token_id: u64) -> FunctionCall {
    use alloy::json_abi::JsonAbi;

    // Minimal NFT marketplace ABI
    let abi: JsonAbi = serde_json::from_str(
        r#"[{
        "type": "function",
        "name": "purchaseNft",
        "stateMutability": "nonpayable",
        "inputs": [{"name": "tokenId", "type": "uint256"}],
        "outputs": []
    }]"#,
    )
    .unwrap();

    FunctionCall {
        target: nft_marketplace_address(),
        abi,
        function_name: "purchaseNft".to_string(),
        args: vec![alloy::dyn_abi::DynSolValue::Uint(
            U256::from(token_id),
            256,
        )],
        value: None,
    }
}
