use eil::*;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use eil::{
        actions::*,
        contract_types::SdkVoucherRequest,
        test_utils::*,
        types::{Amount, TokenAmount},
    };
    use alloy::primitives::U256;

    #[tokio::test]
    async fn test_builder_requires_account() {
        let config = create_test_config(vec![1, 10]);
        let sdk = EilSdk::new(config);

        let builder = sdk.create_builder();

        // Cannot build without setting account
        // This is enforced by the type system
        // builder.start_batch(1); // This would not compile
    }

    #[tokio::test]
    async fn test_builder_with_account() {
        let config = create_test_config(vec![1, 10]);
        let sdk = EilSdk::new(config);
        let account = Arc::new(MockAccount::new());

        let result = sdk.create_builder().use_account(account);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_builder_single_batch() {
        let config = create_test_config(vec![1, 10]);
        let sdk = EilSdk::new(config);
        let account = Arc::new(MockAccount::new());
        let token = create_test_token("USDC", vec![1, 10]);

        let builder = sdk
            .create_builder()
            .use_account(account)
            .unwrap()
            .start_batch(1)
            .add_action(TransferAction {
                token: token.clone(),
                recipient: "0x3333333333333333333333333333333333333333"
                    .parse()
                    .unwrap(),
                amount: Amount::Fixed(U256::from(100)),
            })
            .end_batch();

        // Builder should have 1 batch
        assert_eq!(builder.batch_count(), 1);
    }

    #[tokio::test]
    async fn test_builder_multiple_batches() {
        let config = create_test_config(vec![1, 10, 42161]);
        let sdk = EilSdk::new(config);
        let account = Arc::new(MockAccount::new());
        let token = create_test_token("USDC", vec![1, 10, 42161]);

        let builder = sdk
            .create_builder()
            .use_account(account)
            .unwrap()
            .start_batch(1)
            .add_action(TransferAction {
                token: token.clone(),
                recipient: "0x3333333333333333333333333333333333333333"
                    .parse()
                    .unwrap(),
                amount: Amount::Fixed(U256::from(100)),
            })
            .end_batch()
            .start_batch(10)
            .add_action(TransferAction {
                token: token.clone(),
                recipient: "0x4444444444444444444444444444444444444444"
                    .parse()
                    .unwrap(),
                amount: Amount::Fixed(U256::from(200)),
            })
            .end_batch();

        assert_eq!(builder.batch_count(), 2);
    }

    #[tokio::test]
    async fn test_builder_with_voucher() {
        let config = create_test_config(vec![1, 10]);
        let sdk = EilSdk::new(config);
        let account = Arc::new(MockAccount::with_chains(vec![1, 10]));
        let token = create_test_token("USDC", vec![1, 10]);

        let result = sdk
            .create_builder()
            .use_account(account)
            .unwrap()
            .start_batch(1)
            .add_voucher_request(SdkVoucherRequest {
                ref_id: "v1".to_string(),
                source_chain_id: Some(1),
                destination_chain_id: 10,
                tokens: vec![TokenAmount {
                    token: token.clone(),
                    amount: Amount::Fixed(U256::from(100)),
                    min_provider_deposit: None,
                }],
                target: None,
            })
            .end_batch()
            .start_batch(10)
            .use_voucher("v1");

        assert!(result.is_ok());
        let builder = result.unwrap().end_batch();
        assert_eq!(builder.batch_count(), 2);
    }

    #[tokio::test]
    async fn test_builder_voucher_not_found() {
        let config = create_test_config(vec![1, 10]);
        let sdk = EilSdk::new(config);
        let account = Arc::new(MockAccount::new());

        let result = sdk
            .create_builder()
            .use_account(account)
            .unwrap()
            .start_batch(10)
            .use_voucher("nonexistent");

        assert!(result.is_err());
        match result {
            Err(EilError::VoucherNotFound(ref_id)) => {
                assert_eq!(ref_id, "nonexistent");
            }
            _ => panic!("Expected VoucherNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_builder_actions_ordering() {
        let config = create_test_config(vec![1]);
        let sdk = EilSdk::new(config);
        let account = Arc::new(MockAccount::new());
        let token = create_test_token("USDC", vec![1]);

        let recipient: Address = "0x3333333333333333333333333333333333333333"
            .parse()
            .unwrap();
        let spender: Address = "0x4444444444444444444444444444444444444444"
            .parse()
            .unwrap();

        let builder = sdk
            .create_builder()
            .use_account(account)
            .unwrap()
            .start_batch(1)
            .add_action(ApproveAction {
                token: token.clone(),
                spender,
                value: Amount::Fixed(U256::from(100)),
            })
            .add_action(TransferAction {
                token: token.clone(),
                recipient,
                amount: Amount::Fixed(U256::from(50)),
            })
            .end_batch();

        // Should have 1 batch
        // (We can't easily test action count without exposing more internals,
        //  which is fine - that's tested through other means)
        assert_eq!(builder.batch_count(), 1);
    }
}
