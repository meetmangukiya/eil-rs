# EIL Rust SDK - Implementation Specification

## Overview

This document specifies the Rust implementation of the Ethereum Interoperability Layer (EIL) SDK, designed to provide type-safe, ergonomic cross-chain operations on Ethereum L2s.

## Architecture Summary

### Core Concepts

1. **Trustless Cross-Chain Operations**: Users sign once and execute across multiple L2s without trusting third parties
2. **Voucher System**: Cross-chain token transfers via Cross-chain Liquidity Providers (XLPs)
3. **Account Abstraction**: ERC-4337 UserOperations for flexible smart account support
4. **Batch Operations**: Group multiple actions per chain for gas efficiency
5. **Runtime Variables**: Dynamic values computed on-chain during execution

## Module Structure

### 1. Core Types (`src/types.rs`)

**Status**: ✅ Implemented

Core type definitions:
- `ChainId`: Type alias for `u64`
- `Address`: Re-export from alloy
- `Amount`: Enum for `Fixed(U256)` or `Runtime(RuntimeVar)`
- `RuntimeVar`: Named variable for on-chain computed values
- `Call`, `FunctionCall`: Contract interaction structures
- `OperationStatus`: Execution status tracking

### 2. Error Handling (`src/error.rs`)

**Status**: ✅ Implemented

Comprehensive error types using `thiserror`:
- Chain/address validation errors
- Voucher management errors
- Builder state errors
- Contract interaction errors
- XLP selection errors
- Execution errors

### 3. Configuration (`src/config.rs`)

**Status**: ✅ Implemented

Configuration types:
- `ChainInfo`: Per-chain RPC, EntryPoint, Paymaster
- `XlpSelectionConfig`: XLP filtering and selection policy
- `FeeConfig`: Voucher fee parameters
- `CrossChainConfig`: Main SDK configuration

Features:
- Builder pattern for ergonomic config
- Serde support for JSON config files
- Default values for common settings

### 4. Contract Types (`src/contract_types.rs`)

**Status**: ✅ Implemented

ERC-4337 and EIL contract structures:
- `UserOperation`: ERC-4337 UserOp with all fields
- `VoucherRequest`: Source and destination swap components
- `Asset`: ERC20 token + amount pair
- `AtomicSwapFeeRule`: Fee calculation parameters
- `SdkVoucherRequest`: High-level voucher API
- `SingleChainBatch`: Batch + UserOp + voucher tracking
- `BatchStatusInfo`: Execution state tracking

### 5. Multichain Abstractions (`src/multichain.rs`)

**Status**: ✅ Implemented

Multi-chain contract abstractions:
- `MultichainContract`: Contract deployed across chains
- `MultichainToken`: ERC20 token across chains with standard ABI
- `MultiChainEntity` trait: Unified address resolution
- `AddressPerChain`: HashMap of chain → address mappings

### 6. Actions (`src/actions.rs`)

**Status**: ✅ Implemented (with minor encoding TODO)

Action trait and implementations:
- `Action` trait: Async encode_call method
- `TransferAction`: ERC20 token transfers
- `ApproveAction`: ERC20 approvals
- `FunctionCallAction`: Generic contract calls
- `VoucherRequestAction`: Create voucher requests
- `SetVarAction`: Set runtime variables

**TODO**: Complete ABI encoding using alloy's sol! macro or manual selector encoding

### 7. Voucher System (`src/voucher.rs`)

**Status**: ✅ Implemented

Voucher coordination and XLP management:
- `VoucherCoordinator`: Tracks vouchers across batches
- `InternalVoucherInfo`: Internal voucher state
- `SolventXlpInfo`: XLP liquidity information
- `get_solvent_xlps()`: XLP querying (placeholder)

**TODO**: Implement actual XLP querying from paymaster contracts

### 8. Account Abstraction (`src/account.rs`)

**Status**: ✅ Implemented (interface only)

Multi-chain smart account interface:
- `MultiChainSmartAccount` trait: Core account operations
- `BaseMultichainSmartAccount`: Helper implementation
- `Signer` trait: UserOp signing
- `BundlerManager` trait: UserOp submission

**TODO**: Implement concrete account types (Safe, Biconomy, etc.)

### 9. Network Layer (`src/network.rs`)

**Status**: ⚠️ Implemented (needs alloy API fix)

Network environment management:
- `NetworkEnvironment`: Providers for all chains
- Chain configuration access
- EntryPoint/Paymaster address resolution

**TODO**: Fix alloy `on_http` API usage for HTTP providers

### 10. Builder Pattern (`src/builder.rs`)

**Status**: ✅ Implemented

Type-safe builder pattern with compile-time state tracking:
- `CrossChainBuilder<State>`: Main builder with type-states
  - `Building`: Initial state
  - `ReadyToBuild`: After `use_account()`
  - `Signed`: After signing (implicitly in executor)
- `BatchBuilder`: Single-chain batch builder

Features:
- Fluent API matching TypeScript SDK
- Compile-time prevention of invalid state transitions
- Voucher tracking across batches
- XLP selection and voucher building

**TODO**: Complete UserOp gas estimation

### 11. Executor (`src/executor.rs`)

**Status**: ✅ Implemented (core logic)

Cross-chain execution engine:
- `CrossChainExecutor`: Executes signed UserOps
- `CallbackType`: Execution event types
- `ExecCallback`: User callback for progress
- Execution loop with voucher waiting
- Event-driven batch execution

**TODO**:
- Implement actual bundler submission
- Event polling for voucher signatures
- Transaction confirmation waiting

### 12. Main SDK (`src/lib.rs`)

**Status**: ✅ Implemented

Main SDK entry point:
- `EilSdk`: Primary public API
- `create_builder()`: Start building operations
- `create_token()`: Create multichain tokens
- Re-exports for ergonomic imports

### 13. Utilities (`src/utils.rs`)

**Status**: ✅ Implemented

Helper functions:
- `now_seconds()`: Unix timestamp
- `fee_percent_to_numerator()`: Fee conversion

## API Design

### Type-Safe Builder Pattern

The SDK uses Rust's type system to prevent invalid operations:

```rust
// This won't compile - must call use_account() first
let builder = sdk.create_builder()
    .start_batch(chain_id)  // ❌ Error: method not found

// Correct usage
let executor = sdk.create_builder()
    .use_account(account)?      // Transitions to ReadyToBuild
    .start_batch(chain_id)       // ✅ Now available
        .add_action(action)
        .end_batch()
    .build_and_sign().await?;    // Returns executor
```

### Ergonomic Voucher Management

```rust
.start_batch(ChainId::Optimism)
    .add_voucher_request(VoucherRequest {
        ref_id: "voucher1".into(),
        destination_chain_id: ChainId::Arbitrum,
        tokens: vec![TokenAmount { /* ... */ }],
        ..Default::default()
    })
    .end_batch()
.start_batch(ChainId::Arbitrum)
    .use_voucher("voucher1")?    // Automatic validation
    .add_action(/* ... */)
    .end_batch()
```

### Callback-Based Execution

```rust
executor.execute(|event| {
    match event.callback_type {
        CallbackType::Executing => println!("Executing batch {}...", event.index),
        CallbackType::Done => println!("✓ Batch {} complete", event.index),
        CallbackType::Failed => eprintln!("✗ Batch {} failed", event.index),
        // ...
    }
}).await?;
```

## Implementation Status

### ✅ Completed

1. Core type system with Amount enum
2. Comprehensive error handling
3. Configuration system with builder pattern
4. Contract type abstractions (UserOp, Voucher, etc.)
5. Multichain token/contract abstractions
6. Action trait and basic actions
7. Voucher coordinator
8. Builder pattern with type-states
9. Executor core logic
10. MultiChainSmartAccount trait
11. Main SDK API

### ⚠️ Partial / Needs Fixes

1. **ABI Encoding** (src/actions.rs)
   - Need to fix `abi_encode_input` calls
   - Options: Use sol! macro or manual encoding

2. **HTTP Provider Creation** (src/network.rs)
   - Fix alloy `on_http` API usage
   - May need to use `RootProvider::new_http(url)`

3. **UserOp Hashing** (src/builder.rs)
   - Implement proper EIP-712 hashing
   - Include chainId and entryPoint in domain

### ❌ TODO - Core Features

1. **XLP Querying** (src/voucher.rs)
   - Query paymaster contract for XLPs
   - Filter by deposit requirements
   - Sort by optimal selection criteria

2. **Bundler Integration** (src/account.rs, src/executor.rs)
   - Implement bundler submission
   - Handle UserOp validation errors
   - Retry logic for failed submissions

3. **Event Polling** (src/executor.rs)
   - Watch for VoucherIssued events
   - Watch for UserOperationEvent
   - Handle event filtering and parsing

4. **Account Implementations** (new module needed)
   - Safe account support
   - Biconomy account support
   - Generic ERC-4337 account

5. **Gas Estimation** (src/builder.rs)
   - Estimate callGasLimit
   - Estimate verificationGasLimit
   - Get current gas prices

6. **Runtime Variables** (src/actions.rs, new module)
   - RuntimeVarsHelper contract integration
   - Variable encoding/decoding
   - SetVar action implementation

### ❌ TODO - Nice to Have

1. **Testing**
   - Unit tests for all modules
   - Integration tests with mock chains
   - Property-based tests for builders

2. **Documentation**
   - API documentation
   - Usage examples
   - Migration guide from TypeScript

3. **Advanced Features**
   - Custom paymaster support
   - Gas payment options
   - Multi-sig account support
   - Batch simulation before execution

4. **Optimizations**
   - Parallel batch building
   - Caching of contract calls
   - Efficient event polling

## Dependencies

```toml
[dependencies]
alloy = { version = "0.8", features = ["providers", "rpc-types", "rpc-types-eth", "signer-local", "contract", "json-abi"], default-features = false }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
anyhow = "1.0"
async-trait = "0.1"
hex = "0.4"
once_cell = "1.19"
```

## Next Steps

### Immediate (to get basic compilation):

1. Fix alloy HTTP provider creation
2. Fix ABI encoding in actions
3. Add placeholder implementations for TODO methods

### Short-term (for basic functionality):

1. Implement concrete account type (e.g., SimpleAccount)
2. Add bundler HTTP client
3. Implement basic event polling
4. Add gas estimation

### Long-term (for production):

1. Comprehensive testing
2. Multiple account implementations
3. Advanced XLP selection
4. Runtime variables support
5. Performance optimizations

## Design Decisions

### Why Type-State Pattern?

The type-state pattern prevents invalid builder states at compile time:
- Can't call `start_batch()` before `use_account()`
- Can't call `build_and_sign()` multiple times
- Clear API progression enforced by compiler

### Why Arc for NetworkEnvironment?

`NetworkEnvironment` is shared across builders and executors:
- Avoids cloning providers
- Thread-safe sharing of RPC clients
- Efficient for concurrent operations

### Why Async Trait for Actions?

Actions may need to:
- Query contract state (runtime vars)
- Fetch on-chain data
- Estimate gas

Async allows flexibility without blocking.

### Why Clone for Vouchers?

Voucher coordination needs to avoid borrow checker complexity:
- Clone voucher data when building
- Simpler than lifetimes across builder methods
- Performance impact minimal (small structs)

## Comparison with TypeScript SDK

| Feature | TypeScript | Rust | Notes |
|---------|-----------|------|-------|
| Type Safety | Runtime | Compile-time | Rust catches more errors early |
| Builder State | Runtime checks | Type-states | Rust enforces valid transitions |
| Error Handling | Exceptions | Result<T, E> | Rust forces error handling |
| Memory Safety | GC | Ownership | Rust eliminates memory issues |
| Async | Promises | async/await | Similar ergonomics |
| Performance | V8 JIT | Native | Rust ~10-100x faster |
| Ecosystem | npm | crates.io | Both mature |
| Learning Curve | Lower | Higher | Rust requires understanding ownership |

## Example Usage

```rust
use eil::{EilSdk, config::*, actions::*, types::*};

#[tokio::main]
async fn main() -> eil::Result<()> {
    // Configure SDK
    let config = CrossChainConfig::new(vec![
        ChainInfo { chain_id: 10, rpc_url: "...", /* ... */ },
        ChainInfo { chain_id: 42161, rpc_url: "...", /* ... */ },
    ]);

    let sdk = EilSdk::new(config);
    let usdc = sdk.create_token("USDC", usdc_deployments);

    // Build and execute cross-chain operation
    let executor = sdk.create_builder()
        .use_account(account)?
        .start_batch(10)  // Optimism
            .add_voucher_request(VoucherRequest {
                ref_id: "v1".into(),
                destination_chain_id: 42161,
                tokens: vec![TokenAmount {
                    token: usdc.clone(),
                    amount: Amount::Fixed(U256::from(90_000000)),
                    min_provider_deposit: None,
                }],
                target: None,
            })
            .end_batch()
        .start_batch(42161)  // Arbitrum
            .use_voucher("v1")?
            .add_action(ApproveAction { /* ... */ })
            .add_action(FunctionCallAction { /* ... */ })
            .end_batch()
        .build_and_sign().await?;

    executor.execute(|event| {
        println!("Event: {:?}", event.callback_type);
    }).await?;

    Ok(())
}
```

## Conclusion

The Rust SDK implementation provides a solid foundation with:
- ✅ Type-safe builder pattern
- ✅ Comprehensive error handling
- ✅ Clean module structure
- ✅ Ergonomic API design

The main remaining work is:
- Fixing compilation errors (ABI encoding, HTTP providers)
- Implementing concrete integrations (bundler, events, accounts)
- Adding tests and documentation

The architecture is designed to be extensible and maintainable, with clear separation of concerns and strong type safety throughout.
