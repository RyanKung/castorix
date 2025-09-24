# Castorix Integration Tests

This directory contains integration tests for the Castorix Farcaster protocol integration tool.

## Test Structure

### Rust Tests
- `farcaster_cli_integration_test.rs` - CLI integration tests for basic commands
- `payment_wallet_integration_test.rs` - Payment wallet integration tests
- `base_complete_workflow_test.rs` - Base ENS workflow tests
- `ens_complete_workflow_test.rs` - ENS workflow tests
- `comprehensive_validation_test.rs` - Comprehensive CLI validation tests

### Python Integration Tests
- `test_complete_farcaster_workflow.py` - Complete Farcaster workflow test with interactive CLI handling

## Running Python Integration Tests

### Prerequisites

1. **Install Python dependencies**:
   ```bash
   pip install -r tests/requirements.txt
   ```

2. **Install Foundry** (for Anvil):
   ```bash
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   ```

3. **Ensure Castorix is built**:
   ```bash
   cargo build --release
   ```

### Running the Complete Workflow Test

```bash
cd tests
python test_complete_farcaster_workflow.py
```

## Test Coverage

The Python integration test covers:

1. **Wallet Creation** - Interactive encrypted wallet generation
2. **FID Registration** - Complete FID registration workflow
3. **Storage Rental** - Storage unit rental for FIDs
4. **Signer Management** - Ed25519 signer registration and deletion
5. **Query Operations** - FID listing and storage usage queries

## Key Features

### Interactive CLI Handling
- Uses `pexpect` library for reliable interactive input automation
- Handles complex multi-step CLI interactions
- Proper timeout and error handling

### Complete Workflow Testing
- Tests the full Farcaster protocol integration
- Validates end-to-end functionality
- Ensures wallet creation logic is thoroughly tested

### Environment Management
- Automatic Anvil node startup and shutdown
- Clean test data directory management
- Proper cleanup after test completion

## Test Output

The test provides detailed output showing:
- ✅ Successful operations
- ❌ Failed operations with error details
- 📝 Key information (addresses, FIDs, etc.)
- 📊 Price and usage information

## Troubleshooting

### Common Issues

1. **pexpect not found**:
   ```bash
   pip install pexpect
   ```

2. **anvil not found**:
   ```bash
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   ```

3. **Port 8545 already in use**:
   - Kill existing Anvil processes
   - Or modify the port in the test script

4. **Test timeout**:
   - Increase timeout values in the test script
   - Check if Anvil is running properly

### Debug Mode

To see detailed pexpect output, the test automatically logs all CLI interactions to stdout.
