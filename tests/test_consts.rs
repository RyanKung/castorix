/// Test-specific configuration module
/// This module is the ONLY place allowed to use env::set_var in tests
/// It sets up local test environment variables for RPC URLs

use std::env;

/// Set up local test environment for Anvil node
pub fn setup_local_test_env() {
    // Set local Anvil RPC URLs for testing
    env::set_var("ETH_OP_RPC_URL", "http://127.0.0.1:8545");
    env::set_var("ETH_RPC_URL", "http://127.0.0.1:8545");
    env::set_var("ETH_BASE_RPC_URL", "http://127.0.0.1:8545");
    env::set_var("FARCASTER_HUB_URL", "http://192.168.1.192:3381");
}

/// Set up placeholder URLs for configuration validation testing
pub fn setup_placeholder_test_env() {
    env::set_var("ETH_OP_RPC_URL", "https://www.optimism.io/");
    env::set_var("ETH_RPC_URL", "https://eth-mainnet.g.alchemy.com/v2/your_api_key_here");
    env::set_var("ETH_BASE_RPC_URL", "https://mainnet.base.org");
    env::set_var("FARCASTER_HUB_URL", "http://192.168.1.192:3381");
}

/// Set up demo API URLs for simple testing
pub fn setup_demo_test_env() {
    env::set_var("ETH_OP_RPC_URL", "https://optimism-mainnet.g.alchemy.com/v2/demo");
    env::set_var("ETH_RPC_URL", "https://eth-mainnet.g.alchemy.com/v2/demo");
    env::set_var("ETH_BASE_RPC_URL", "https://mainnet.base.org");
    env::set_var("FARCASTER_HUB_URL", "https://hub-api.neynar.com");
}

/// Reset environment to default values
pub fn reset_test_env() {
    env::remove_var("ETH_OP_RPC_URL");
    env::remove_var("ETH_RPC_URL");
    env::remove_var("ETH_BASE_RPC_URL");
    env::remove_var("FARCASTER_HUB_URL");
}

/// Check if we should skip RPC tests
pub fn should_skip_rpc_tests() -> bool {
    env::var("SKIP_RPC_TESTS").is_ok()
}
