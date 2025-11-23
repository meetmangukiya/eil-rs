#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;

    #[test]
    fn test_amount_from_u256() {
        let amount = Amount::from(U256::from(100));
        match amount {
            Amount::Fixed(val) => assert_eq!(val, U256::from(100)),
            Amount::Runtime(_) => panic!("Expected Fixed amount"),
        }
    }

    #[test]
    fn test_amount_from_u64() {
        let amount = Amount::from(42u64);
        match amount {
            Amount::Fixed(val) => assert_eq!(val, U256::from(42)),
            Amount::Runtime(_) => panic!("Expected Fixed amount"),
        }
    }

    #[test]
    fn test_runtime_var_valid() {
        let var = RuntimeVar::new("myvar").unwrap();
        assert_eq!(var.name, "myvar");
    }

    #[test]
    fn test_runtime_var_max_length() {
        let var = RuntimeVar::new("12345678").unwrap();
        assert_eq!(var.name, "12345678");
    }

    #[test]
    fn test_runtime_var_too_long() {
        let result = RuntimeVar::new("123456789");
        assert!(result.is_err());
        match result {
            Err(crate::EilError::InvalidVariableName(name)) => {
                assert_eq!(name, "123456789");
            }
            _ => panic!("Expected InvalidVariableName error"),
        }
    }

    #[test]
    fn test_runtime_var_empty() {
        let var = RuntimeVar::new("").unwrap();
        assert_eq!(var.name, "");
    }

    #[test]
    fn test_amount_from_runtime_var() {
        let var = RuntimeVar::new("test").unwrap();
        let amount = Amount::from(var.clone());
        match amount {
            Amount::Runtime(v) => assert_eq!(v.name, "test"),
            Amount::Fixed(_) => panic!("Expected Runtime amount"),
        }
    }

    #[test]
    fn test_operation_status() {
        assert_eq!(OperationStatus::Pending, OperationStatus::Pending);
        assert_ne!(OperationStatus::Pending, OperationStatus::Done);
    }

    #[test]
    fn test_chain_ids_constants() {
        use crate::types::chain_ids::*;
        assert_eq!(MAINNET, 1);
        assert_eq!(OPTIMISM, 10);
        assert_eq!(ARBITRUM, 42161);
        assert_eq!(BASE, 8453);
        assert_eq!(POLYGON, 137);
    }
}
