#!/bin/bash

# Script to start Anvil for local blockchain testing
# This script starts a local Ethereum node for testing transaction sending

echo "🚀 Starting Anvil local blockchain..."

# Check if anvil is installed
if ! command -v anvil &> /dev/null; then
    echo "❌ Anvil not found. Please install Foundry first:"
    echo "   curl -L https://foundry.paradigm.xyz | bash"
    echo "   foundryup"
    exit 1
fi

# Start Anvil with some pre-funded accounts
anvil \
    --host 0.0.0.0 \
    --port 8545 \
    --accounts 10 \
    --balance 10000 \
    --gas-limit 30000000 \
    --gas-price 1000000000 \
    --chain-id 31337 \
    --fork-url https://optimism-mainnet.infura.io/v3/your-api-key \
    --fork-block-number 12345678 \
    --verbose

echo "✅ Anvil started successfully!"
echo "   RPC URL: http://localhost:8545"
echo "   Chain ID: 31337"
echo "   Pre-funded accounts: 10"
echo ""
echo "To run tests with real blockchain:"
echo "   cargo test transaction_tests"
echo ""
echo "To stop Anvil: Ctrl+C"
