#!/bin/bash

# Test script with Anvil setup and teardown
# Usage: ./scripts/test-with-anvil.sh [test_args...]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ANVIL_PORT=8545
ANVIL_PID_FILE="/tmp/anvil-test-${ANVIL_PORT}.pid"
TEST_ARGS="$@"

echo -e "${BLUE}🚀 Starting test environment setup...${NC}"

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up test environment...${NC}"
    if [ -f "$ANVIL_PID_FILE" ]; then
        ANVIL_PID=$(cat "$ANVIL_PID_FILE")
        if kill -0 "$ANVIL_PID" 2>/dev/null; then
            echo -e "${YELLOW}   Stopping Anvil (PID: $ANVIL_PID)${NC}"
            kill "$ANVIL_PID"
            rm -f "$ANVIL_PID_FILE"
        fi
    fi
}

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

# Check if Anvil is already running
check_anvil() {
    curl -s -o /dev/null -w "%{http_code}" "http://localhost:$ANVIL_PORT" | grep -q "200\|405\|400"
}

# Start Anvil if not running
start_anvil() {
    if check_anvil; then
        echo -e "${GREEN}✅ Anvil is already running on localhost:$ANVIL_PORT${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}🔧 Starting Anvil on localhost:$ANVIL_PORT...${NC}"
    
    # Check if anvil command exists
    if ! command -v anvil &> /dev/null; then
        echo -e "${RED}❌ Anvil not found. Please install Foundry:${NC}"
        echo "   curl -L https://foundry.paradigm.xyz | bash"
        echo "   foundryup"
        exit 1
    fi
    
    # Start Anvil in background
    anvil \
        --host 0.0.0.0 \
        --port $ANVIL_PORT \
        --accounts 10 \
        --balance 10000 \
        --gas-limit 30000000 \
        --gas-price 1000000000 \
        --chain-id 31337 \
        --silent \
        > /tmp/anvil-test.log 2>&1 &
    
    ANVIL_PID=$!
    echo $ANVIL_PID > "$ANVIL_PID_FILE"
    
    # Wait for Anvil to be ready
    echo -e "${YELLOW}   Waiting for Anvil to be ready...${NC}"
    for i in {1..30}; do
        if check_anvil; then
            echo -e "${GREEN}✅ Anvil is ready!${NC}"
            return 0
        fi
        sleep 1
    done
    
    echo -e "${RED}❌ Anvil failed to start within 30 seconds${NC}"
    echo "   Check /tmp/anvil-test.log for details"
    exit 1
}

# Run tests
run_tests() {
    echo -e "${BLUE}🧪 Running tests...${NC}"
    
    if [ -n "$TEST_ARGS" ]; then
        echo -e "${BLUE}   Test args: $TEST_ARGS${NC}"
        RUNNING_TESTS=1 cargo test $TEST_ARGS
    else
        echo -e "${BLUE}   Running all tests${NC}"
        RUNNING_TESTS=1 cargo test
    fi
    
    TEST_EXIT_CODE=$?
    
    if [ $TEST_EXIT_CODE -eq 0 ]; then
        echo -e "${GREEN}✅ All tests passed!${NC}"
    else
        echo -e "${RED}❌ Some tests failed${NC}"
    fi
    
    return $TEST_EXIT_CODE
}

# Main execution
main() {
    start_anvil
    run_tests
}

# Run main function
main
