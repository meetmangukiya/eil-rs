use eil::*;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use eil::test_utils::*;

    #[test]
    fn test_sdk_creation() {
        let config = create_test_config(vec![1, 10]);
        let sdk = EilSdk::new(config);

        assert_eq!(sdk.network_env().chain_ids().len(), 2);
    }

    #[test]
    fn test_create_token() {
        let config = create_test_config(vec![1, 10]);
        let sdk = EilSdk::new(config);

        let token = sdk.create_token("USDC", create_test_token("USDC", vec![1, 10]).deployments);

        assert_eq!(token.name, "USDC");
        assert!(token.is_deployed_on(1));
        assert!(token.is_deployed_on(10));
        assert!(!token.is_deployed_on(42161));
    }

    #[test]
    fn test_builder_creation() {
        let config = create_test_config(vec![1, 10]);
        let sdk = EilSdk::new(config);

        let _builder = sdk.create_builder();
        // Builder created successfully
    }

    #[tokio::test]
    async fn test_simple_cross_chain_flow() {
        let config = create_test_config(vec![1, 10]);
        let sdk = EilSdk::new(config);
        let account = Arc::new(MockAccount::new());

        // This should compile but will have placeholder implementations
        let result = sdk
            .create_builder()
            .use_account(account.clone());

        assert!(result.is_ok());
    }

    #[test]
    fn test_network_environment_rpc_urls() {
        let config = create_test_config(vec![1, 10, 42161]);
        let env = network::NetworkEnvironment::new(&config);

        assert!(env.rpc_url(1).is_ok());
        assert!(env.rpc_url(10).is_ok());
        assert!(env.rpc_url(42161).is_ok());
        assert!(env.rpc_url(999).is_err());
    }

    #[test]
    fn test_network_environment_entry_points() {
        let config = create_test_config(vec![1, 10]);
        let env = network::NetworkEnvironment::new(&config);

        assert!(env.entry_point(1).is_ok());
        assert!(env.entry_point(10).is_ok());
        assert!(env.entry_point(999).is_err());
    }

    #[test]
    fn test_network_environment_paymasters() {
        let config = create_test_config(vec![1, 10]);
        let env = network::NetworkEnvironment::new(&config);

        assert!(env.paymaster(1).is_ok());
        assert!(env.paymaster(10).is_ok());
        assert!(env.paymaster(999).is_err());
    }
}
