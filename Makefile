# Castorix Local Development Makefile
# This Makefile provides commands for local development and testing

.PHONY: help install build test test-local test-ci clean start-nodes stop-nodes status-nodes

# Default target
help:
	@echo "Castorix Development Commands:"
	@echo ""
	@echo "Node Management:"
	@echo "  start-nodes     - Start local Anvil nodes for testing"
	@echo "  stop-nodes      - Stop all running Anvil nodes"
	@echo "  status-nodes    - Check status of Anvil nodes"
	@echo ""
	@echo "Development:"
	@echo "  install         - Install dependencies and tools"
	@echo "  build           - Build the project"
	@echo "  test            - Run tests with pre-started nodes"
	@echo "  test-local      - Start nodes and run tests locally"
	@echo "  test-ci         - Run tests in CI mode (expects pre-started nodes)"
	@echo "  clean           - Clean build artifacts and test data"
	@echo ""
	@echo "Quick Commands:"
	@echo "  dev             - Start nodes and run tests (alias for test-local)"
	@echo "  ci              - Run tests in CI mode (alias for test-ci)"

# Install dependencies and tools
install:
	@echo "🔧 Installing dependencies and tools..."
	@if ! command -v anvil >/dev/null 2>&1; then \
		echo "Installing Foundry (includes anvil)..."; \
		curl -L https://foundry.paradigm.xyz | bash; \
		foundryup; \
	fi
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "Installing Rust..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		source ~/.cargo/env; \
	fi
	@echo "✅ Dependencies installed"

# Build the project
build:
	@echo "🔨 Building Castorix..."
	cargo build --all-features
	@echo "✅ Build completed"

# Start local Anvil nodes for testing
start-nodes:
	@echo "🚀 Starting local Anvil nodes for testing..."
	@echo "Starting Optimism Anvil node on port 8545..."
	@anvil --fork-url "https://mainnet.optimism.io" --port 8545 --host 127.0.0.1 --block-time 1 --retries 3 --timeout 10000 > /tmp/anvil-optimism.log 2>&1 &
	@echo $$! > /tmp/anvil-optimism.pid
	@echo "Starting Base Anvil node on port 8546..."
	@anvil --fork-url "https://base-rpc.publicnode.com" --port 8546 --host 127.0.0.1 --block-time 1 --retries 3 --timeout 10000 > /tmp/anvil-base.log 2>&1 &
	@echo $$! > /tmp/anvil-base.pid
	@echo "⏳ Waiting for nodes to start..."
	@sleep 5
	@echo "✅ Anvil nodes started"
	@echo "Optimism node PID: $$(cat /tmp/anvil-optimism.pid)"
	@echo "Base node PID: $$(cat /tmp/anvil-base.pid)"
	@echo ""
	@echo "Logs:"
	@echo "  Optimism: tail -f /tmp/anvil-optimism.log"
	@echo "  Base: tail -f /tmp/anvil-base.log"

# Stop all running Anvil nodes
stop-nodes:
	@echo "🛑 Stopping Anvil nodes..."
	@if [ -f /tmp/anvil-optimism.pid ]; then \
		kill $$(cat /tmp/anvil-optimism.pid) 2>/dev/null || true; \
		rm -f /tmp/anvil-optimism.pid; \
		echo "✅ Optimism Anvil stopped"; \
	else \
		echo "ℹ️  No Optimism Anvil PID file found"; \
	fi
	@if [ -f /tmp/anvil-base.pid ]; then \
		kill $$(cat /tmp/anvil-base.pid) 2>/dev/null || true; \
		rm -f /tmp/anvil-base.pid; \
		echo "✅ Base Anvil stopped"; \
	else \
		echo "ℹ️  No Base Anvil PID file found"; \
	fi
	@# Also kill any remaining anvil processes
	@pkill -f "anvil.*8545" 2>/dev/null || true
	@pkill -f "anvil.*8546" 2>/dev/null || true
	@echo "🧹 Cleanup completed"

# Check status of Anvil nodes
status-nodes:
	@echo "📊 Checking Anvil node status..."
	@echo ""
	@if [ -f /tmp/anvil-optimism.pid ]; then \
		PID=$$(cat /tmp/anvil-optimism.pid); \
		if ps -p $$PID > /dev/null 2>&1; then \
			echo "✅ Optimism Anvil (PID: $$PID) - Running on port 8545"; \
		else \
			echo "❌ Optimism Anvil (PID: $$PID) - Not running"; \
		fi; \
	else \
		echo "ℹ️  Optimism Anvil - No PID file found"; \
	fi
	@if [ -f /tmp/anvil-base.pid ]; then \
		PID=$$(cat /tmp/anvil-base.pid); \
		if ps -p $$PID > /dev/null 2>&1; then \
			echo "✅ Base Anvil (PID: $$PID) - Running on port 8546"; \
		else \
			echo "❌ Base Anvil (PID: $$PID) - Not running"; \
		fi; \
	else \
		echo "ℹ️  Base Anvil - No PID file found"; \
	fi
	@echo ""
	@echo "Testing connectivity..."
	@curl -s -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' http://127.0.0.1:8545 >/dev/null && echo "✅ Optimism node responding" || echo "❌ Optimism node not responding"
	@curl -s -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' http://127.0.0.1:8546 >/dev/null && echo "✅ Base node responding" || echo "❌ Base node not responding"

# Run tests with pre-started nodes (CI mode)
test-ci:
	@echo "🧪 Running tests in CI mode (expects pre-started nodes)..."
	@export ETH_OP_RPC_URL="http://127.0.0.1:8545"; \
	export ETH_BASE_RPC_URL="http://127.0.0.1:8546"; \
	export RUNNING_TESTS="true"; \
	cargo test --test ens_complete_workflow_test --verbose; \
	cargo test --test base_complete_workflow_test --verbose
	@echo "✅ CI tests completed"

# Start nodes and run tests locally
test-local: start-nodes
	@echo "🧪 Running local tests with started nodes..."
	@export ETH_OP_RPC_URL="http://127.0.0.1:8545"; \
	export ETH_BASE_RPC_URL="http://127.0.0.1:8546"; \
	export RUNNING_TESTS="true"; \
	cargo test --test ens_complete_workflow_test --verbose; \
	cargo test --test base_complete_workflow_test --verbose
	@echo "✅ Local tests completed"
	@$(MAKE) stop-nodes

# Run tests (default: CI mode)
test: test-ci

# Alias for test-local
dev: test-local

# Alias for test-ci  
ci: test-ci

# Clean build artifacts and test data
clean:
	@echo "🧹 Cleaning build artifacts and test data..."
	cargo clean
	rm -rf ./test_ens_data
	rm -rf ./test_base_data
	rm -rf ./test_farcaster_data
	rm -f /tmp/anvil-*.log
	rm -f /tmp/anvil-*.pid
	@echo "✅ Cleanup completed"

# Quick development setup
setup: install build start-nodes
	@echo "🎉 Development environment ready!"
	@echo ""
	@echo "Available commands:"
	@echo "  make test-local  - Run tests with local nodes"
	@echo "  make test-ci     - Run tests in CI mode"
	@echo "  make status-nodes - Check node status"
	@echo "  make stop-nodes  - Stop all nodes"
