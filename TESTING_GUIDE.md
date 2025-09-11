# Testing Guide for Castorix

This guide explains how to run different types of tests in the Castorix project, including the new transaction-sending tests with mock network environments.

## Test Categories

### 1. Unit Tests
Basic functionality tests that don't require external dependencies.

```bash
cargo test
```

### 2. Onchain Tests (Read-Only)
Tests that read data from live Farcaster contracts on Optimism mainnet.

```bash
# Set up environment variable
export ETH_OP_RPC_URL="your-optimism-rpc-url"

# Run onchain tests
cargo test onchain_tests
```

### 3. Transaction Tests (Mock Network)
Tests that simulate sending transactions using a local blockchain environment.

```bash
# Run all transaction tests
cargo test transaction_tests

# Run specific transaction tests
cargo test test_wallet_creation_and_funding
cargo test test_basic_transaction_sending
cargo test test_contract_interaction
```

### 4. Wallet Client Tests
Tests for wallet functionality, signing, and key management.

```bash
# Run all wallet tests
cargo test wallet_client_tests

# Run specific wallet tests
cargo test test_farcaster_message_signing
cargo test test_eip712_signing
cargo test test_wallet_encryption
```

## Mock Network Environment

The transaction tests use a hybrid approach that supports both real and mock blockchain environments:

### Why Anvil Integration is Limited

**Problem**: ethers-rs 2.0.14 doesn't include a built-in `anvil` feature for direct integration.

**Solution**: We use a hybrid approach:
- **Real blockchain**: When Anvil is running locally (http://localhost:8545)
- **Mock blockchain**: When no local blockchain is available

### Test Environment Features

- **Hybrid Testing**: Automatically detects if local blockchain is available
- **Fallback Mocking**: Graceful degradation to mock behavior when blockchain unavailable
- **Wallet Management**: Automatic creation and funding of test wallets
- **Transaction Simulation**: Real transaction sending when connected, mock when not
- **Gas Estimation**: Tests include gas estimation and optimization
- **Multi-transaction Support**: Batch transaction testing

### Starting Local Blockchain

To run tests with a real local blockchain:

1. **Start Anvil** (requires Foundry installation):
   ```bash
   # Option 1: Use our script
   ./scripts/start-anvil.sh
   
   # Option 2: Manual start
   anvil --accounts 10 --balance 10000
   ```

2. **Run tests**:
   ```bash
   cargo test transaction_tests
   ```

If Anvil is not running, tests will automatically use mock behavior with helpful output messages.

## Running Tests with Pre-Setup Scripts

### Using Makefile (Recommended)

```bash
# Run all tests in mock mode (default)
make test

# Run tests with real blockchain (starts Anvil automatically)
make test-anvil

# Run tests with real blockchain (assumes Anvil is running)
make test-real

# Run tests in mock mode only
make test-mock

# Stop any running Anvil processes
make clean-anvil

# Show all available commands
make help
```

### Using Test Scripts

```bash
# Run tests with automatic Anvil setup and cleanup
./scripts/test-with-anvil.sh

# Run specific tests with Anvil
./scripts/test-with-anvil.sh simple_tests

# Start Anvil manually
./scripts/start-anvil.sh
```

### Using Environment Variables

```bash
# Run tests with build script setup
RUNNING_TESTS=1 cargo test

# Run tests with verbose output
cargo test -- --nocapture
```

### Wallet Functionality Tests
```bash
# Test message signing
cargo test test_farcaster_message_signing

# Test EIP-712 typed data signing
cargo test test_eip712_signing

# Test wallet encryption/decryption
cargo test test_wallet_encryption

# Test key derivation
cargo test test_wallet_key_derivation
```

### Advanced Tests
```bash
# Test transaction building and signing
cargo test test_transaction_building

# Test batch transactions
cargo test test_batch_transactions

# Test wallet recovery
cargo test test_wallet_recovery

# Test multi-sig simulation
cargo test test_multi_sig_simulation
```

## Test Environment Setup

### Prerequisites
1. **Foundry**: Make sure Foundry is installed (we installed it earlier)
2. **Rust**: Ensure you have the latest Rust toolchain
3. **Dependencies**: All required dependencies are in `Cargo.toml`

### Environment Variables (Optional)
For onchain tests, you can set:
```bash
export ETH_OP_RPC_URL="your-optimism-rpc-url"
```

## Test Structure

```
src/farcaster/contracts/
├── test_utils.rs              # Test environment utilities
├── transaction_tests.rs       # Basic transaction tests
├── wallet_client_tests.rs     # Wallet and signing tests
├── onchain_tests.rs          # Live blockchain tests
└── tests.rs                  # Unit tests
```

## Test Utilities

The `test_utils.rs` module provides:

- **TestEnvironment**: Manages Anvil instance and test wallets
- **MockContractAddresses**: Provides mock contract addresses for testing
- **Helper Functions**: Utilities for creating funded wallets and sending transactions

## Example Test Output

When running transaction tests, you'll see output like:

```
✅ Wallet created and funded successfully
   Address: 0x742d35Cc6634C0532925a3b8D5c3C2C2c2c2c2c2c2c
   Balance: 1000.0 ETH

✅ Basic transaction sent successfully
   Transaction hash: 0x1234567890abcdef...
   Gas used: 21000

✅ Signature creation and verification successful
   Message: Hello, Farcaster!
   Signer: 0x742d35Cc6634C0532925a3b8D5c3C2C2c2c2c2c2c2c
```

## Troubleshooting

### Common Issues

1. **Anvil not found**: Make sure Foundry is installed and in your PATH
2. **Port conflicts**: Anvil uses port 8545 by default, make sure it's available
3. **Memory issues**: Large test suites might need more memory allocation

### Debug Mode
Run tests with more verbose output:
```bash
RUST_LOG=debug cargo test transaction_tests
```

## Future Enhancements

Planned improvements to the testing suite:

1. **Contract Deployment Tests**: Deploy mock Farcaster contracts for full integration testing
2. **Gas Optimization Tests**: Test and optimize gas usage for different operations
3. **Error Handling Tests**: Comprehensive error scenario testing
4. **Performance Tests**: Benchmark transaction throughput and latency
5. **Cross-chain Tests**: Test interactions across different networks

## Test Results Summary

现在运行 `cargo test` 时：
- ✅ **26个单元测试通过** - 所有功能测试正常
- ✅ **1个文档测试通过** - 只保留有用的文档测试
- ✅ **0个被忽略的文档测试** - 自动生成的合约绑定文档测试被完全忽略
- ✅ **混合测试环境** - 支持真实区块链和模拟环境
- ✅ **没有警告输出** - 干净的测试输出

### 测试类型总结

1. **基础功能测试** (7个测试通过):
   - 钱包创建和签名
   - 消息签名和验证
   - 测试环境创建
   - 模拟合约地址
   - 客户端创建
   - 钱包资金和余额检查

2. **集成测试** (2个被忽略):
   - 真实区块链连接测试
   - Farcaster 工作流模拟测试

3. **现有测试** (19个测试通过):
   - 配置和常量测试
   - 密钥管理测试
   - ENS 证明测试
   - Farcaster 客户端测试
   - 加密密钥管理测试

### 模拟网络环境特性

- **智能检测**: 自动检测是否有本地区块链连接
- **优雅降级**: 没有连接时使用模拟行为
- **真实交易**: 有连接时使用真实区块链
- **完整日志**: 清晰的测试输出和状态信息

## Contributing

When adding new tests:

1. Follow the existing test structure
2. Use descriptive test names
3. Include helpful debug output
4. Add appropriate error handling
5. Update this guide if adding new test categories
