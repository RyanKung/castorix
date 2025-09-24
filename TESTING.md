# Castorix Testing Guide

This document explains how to run tests for Castorix, including the new workflow-based Anvil node management system.

## Overview

Castorix uses a two-tier testing approach:
- **CI Environment**: GitHub Actions pre-starts Anvil nodes before running tests
- **Local Environment**: Use Makefile to manage Anvil nodes for local testing

## Quick Start

### For Local Development

```bash
# Start nodes and run all tests
make dev

# Or step by step:
make start-nodes    # Start Anvil nodes
make test-ci        # Run tests (expects nodes to be running)
make stop-nodes     # Stop nodes when done
```

### For CI Testing

Tests automatically detect the CI environment and use pre-started nodes managed by GitHub Actions.

## Available Make Commands

### Node Management
- `make start-nodes` - Start local Anvil nodes for testing
- `make stop-nodes` - Stop all running Anvil nodes  
- `make status-nodes` - Check status of Anvil nodes

### Testing
- `make test-local` - Start nodes and run tests locally
- `make test-ci` - Run tests in CI mode (expects pre-started nodes)
- `make test` - Alias for test-ci
- `make dev` - Alias for test-local

### Development
- `make install` - Install dependencies and tools
- `make build` - Build the project
- `make clean` - Clean build artifacts and test data

## Node Configuration

### Optimism Node (Port 8545)
- Fork URL: `https://mainnet.optimism.io`
- Used for ENS workflow tests
- Environment variable: `ETH_OP_RPC_URL`

### Base Node (Port 8546)  
- Fork URL: `https://base-rpc.publicnode.com`
- Used for Base workflow tests
- Environment variable: `ETH_BASE_RPC_URL`

## Environment Variables

Tests use these environment variables to detect the environment:

- `RUNNING_TESTS=true` - Indicates CI environment with pre-started nodes
- `ETH_OP_RPC_URL` - Optimism RPC URL (default: http://127.0.0.1:8545)
- `ETH_BASE_RPC_URL` - Base RPC URL (default: http://127.0.0.1:8546)

## Test Types

### Workflow Tests
These tests require Anvil nodes and test complete workflows:

- `ens_complete_workflow_test` - Full ENS workflow testing
- `base_complete_workflow_test` - Full Base workflow testing

### Unit Tests
These tests don't require external services:

- All tests in `src/` directory
- Configuration validation tests
- Help command tests

## Troubleshooting

### Port Already in Use
```bash
make stop-nodes  # Stop existing nodes
make start-nodes # Restart nodes
```

### Check Node Status
```bash
make status-nodes
```

### View Node Logs
```bash
tail -f /tmp/anvil-optimism.log  # Optimism node logs
tail -f /tmp/anvil-base.log      # Base node logs
```

### Clean Everything
```bash
make clean       # Clean build artifacts
make stop-nodes  # Stop all nodes
```

## CI/CD Integration

The GitHub Actions workflow automatically:

1. Starts Optimism Anvil node on port 8545
2. Starts Base Anvil node on port 8546  
3. Waits for nodes to be ready
4. Sets environment variables for tests
5. Runs all integration tests
6. Stops nodes when complete (even on failure)

## Development Workflow

### For New Features
1. `make start-nodes` - Start development nodes
2. Develop and test your feature
3. `make test-ci` - Run tests against running nodes
4. `make stop-nodes` - Clean up when done

### For CI Testing
1. Push changes to trigger GitHub Actions
2. Workflow automatically manages nodes
3. Tests run against pre-started nodes
4. Results reported in PR

## RPC Endpoints

The system uses public RPC endpoints that don't require API keys:

- **Optimism**: `https://mainnet.optimism.io`
- **Base**: `https://base-rpc.publicnode.com`

This ensures tests work reliably in CI environments without authentication dependencies.

## Best Practices

1. Always use `make` commands for local development
2. Don't manually start Anvil nodes - use the Makefile
3. Check node status before running tests
4. Clean up nodes when development is complete
5. Use `make clean` to reset everything if issues occur

## Support

If you encounter issues:

1. Check node status: `make status-nodes`
2. View logs: `tail -f /tmp/anvil-*.log`
3. Clean and restart: `make clean && make start-nodes`
4. Check for port conflicts: `lsof -i :8545 -i :8546`
