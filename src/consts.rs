use std::env;

/// Environment variable configuration manager
/// Loads configuration from .env file and provides typed access to environment variables
pub struct Config {
    pub eth_rpc_url: String,
    pub eth_base_rpc_url: String,
    pub eth_op_rpc_url: String,
    pub farcaster_hub_url: String,
}

impl Config {
    /// Load configuration from environment variables
    /// This will load from .env file if present, then override with system environment variables
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        if dotenv::dotenv().is_err() {
            // .env file not found, continue with system environment variables
            eprintln!("Warning: .env file not found, using system environment variables only");
        }

        Ok(Self {
            eth_rpc_url: env::var("ETH_RPC_URL").unwrap_or_else(|_| {
                "https://eth-mainnet.g.alchemy.com/v2/your_api_key_here".to_string()
            }),
            eth_base_rpc_url: env::var("ETH_BASE_RPC_URL")
                .unwrap_or_else(|_| "https://base-mainnet.g.alchemy.com/v2/demo".to_string()),
            eth_op_rpc_url: env::var("ETH_OP_RPC_URL")
                .unwrap_or_else(|_| "https://www.optimism.io/".to_string()),
            farcaster_hub_url: env::var("FARCASTER_HUB_URL")
                .unwrap_or_else(|_| "http://192.168.1.192:3381".to_string()),
        })
    }

    /// Load configuration with custom .env file path
    pub fn load_from_file(env_file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::from_path(env_file)?;

        Ok(Self {
            eth_rpc_url: env::var("ETH_RPC_URL").unwrap_or_else(|_| {
                "https://eth-mainnet.g.alchemy.com/v2/your_api_key_here".to_string()
            }),
            eth_base_rpc_url: env::var("ETH_BASE_RPC_URL")
                .unwrap_or_else(|_| "https://base-mainnet.g.alchemy.com/v2/demo".to_string()),
            eth_op_rpc_url: env::var("ETH_OP_RPC_URL")
                .unwrap_or_else(|_| "https://www.optimism.io/".to_string()),
            farcaster_hub_url: env::var("FARCASTER_HUB_URL")
                .unwrap_or_else(|_| "http://192.168.1.192:3381".to_string()),
        })
    }

    /// Validate that all required environment variables are set
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.eth_rpc_url.contains("your_api_key_here") {
            errors.push(
                "ETH_RPC_URL contains placeholder value, please set your actual API key"
                    .to_string(),
            );
        }

        if self.eth_base_rpc_url.is_empty() {
            errors.push("ETH_BASE_RPC_URL is empty".to_string());
        }

        if self.eth_op_rpc_url.is_empty() {
            errors.push("ETH_OP_RPC_URL is empty".to_string());
        }

        if self.farcaster_hub_url.is_empty() {
            errors.push("FARCASTER_HUB_URL is empty".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get Ethereum RPC URL
    pub fn eth_rpc_url(&self) -> &str {
        &self.eth_rpc_url
    }

    /// Get Base chain RPC URL
    pub fn eth_base_rpc_url(&self) -> &str {
        &self.eth_base_rpc_url
    }

    /// Get Optimism RPC URL
    pub fn eth_op_rpc_url(&self) -> &str {
        &self.eth_op_rpc_url
    }

    /// Get Farcaster Hub URL
    pub fn farcaster_hub_url(&self) -> &str {
        &self.farcaster_hub_url
    }

    /// Print current configuration (masking sensitive values)
    pub fn print_config(&self) {
        println!("=== Configuration ===");
        println!("ETH_RPC_URL: {}", mask_url(&self.eth_rpc_url));
        println!("ETH_BASE_RPC_URL: {}", self.eth_base_rpc_url);
        println!("ETH_OP_RPC_URL: {}", self.eth_op_rpc_url);
        println!("FARCASTER_HUB_URL: {}", self.farcaster_hub_url);
        println!("===================");
    }
}

/// Mask sensitive parts of URLs (like API keys)
fn mask_url(url: &str) -> String {
    if url.contains("your_api_key_here") {
        url.to_string()
    } else if let Some(api_key_start) = url.find("/v2/") {
        if api_key_start + 4 < url.len() {
            let masked = format!("{}***", &url[..api_key_start + 4]);
            masked
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

lazy_static::lazy_static! {
    /// Global configuration instance
    /// This is a lazy static that loads configuration once when first accessed
    pub static ref CONFIG: Config = Config::load().expect("Failed to load configuration");
}

/// Get the global configuration instance
pub fn get_config() -> &'static Config {
    &CONFIG
}

/// Initialize configuration and validate
/// Call this at the start of your application
pub fn init_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config();

    match config.validate() {
        Ok(()) => {
            println!("Configuration loaded successfully");
            config.print_config();
            Ok(())
        }
        Err(errors) => {
            eprintln!("Configuration validation failed:");
            for error in errors {
                eprintln!("  - {error}");
            }
            eprintln!("\nPlease check your .env file or environment variables.");
            Err("Configuration validation failed".into())
        }
    }
}

/// Environment variable names as constants
pub mod env_vars {
    pub const ETH_RPC_URL: &str = "ETH_RPC_URL";
    pub const ETH_BASE_RPC_URL: &str = "ETH_BASE_RPC_URL";
    pub const ETH_OP_RPC_URL: &str = "ETH_OP_RPC_URL";
    pub const FARCASTER_HUB_URL: &str = "FARCASTER_HUB_URL";
}

/// Default values for environment variables
pub mod defaults {
    pub const ETH_RPC_URL: &str = "https://eth-mainnet.g.alchemy.com/v2/your_api_key_here";
    pub const ETH_BASE_RPC_URL: &str = "https://base-mainnet.g.alchemy.com/v2/demo";
    pub const ETH_OP_RPC_URL: &str = "https://www.optimism.io/";
    pub const FARCASTER_HUB_URL: &str = "http://192.168.1.192:3381";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = Config::load().expect("Failed to load config");

        // Test that all URLs are non-empty
        assert!(!config.eth_rpc_url.is_empty());
        assert!(!config.eth_base_rpc_url.is_empty());
        assert!(!config.eth_op_rpc_url.is_empty());
        assert!(!config.farcaster_hub_url.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let config = Config::load().expect("Failed to load config");

        // Validation might fail if using placeholder values, which is expected
        let validation_result = config.validate();

        // We just test that validation returns either Ok or Err with messages
        match validation_result {
            Ok(()) => println!("Configuration is valid"),
            Err(errors) => {
                println!(
                    "Configuration validation failed with {} errors",
                    errors.len()
                );
                assert!(!errors.is_empty());
            }
        }
    }

    #[test]
    fn test_url_masking() {
        let url_with_api_key = "https://eth-mainnet.g.alchemy.com/v2/abc123def456";
        let masked = mask_url(url_with_api_key);
        assert!(masked.contains("***"));
        assert!(!masked.contains("abc123def456"));

        let placeholder_url = "https://eth-mainnet.g.alchemy.com/v2/your_api_key_here";
        let not_masked = mask_url(placeholder_url);
        assert_eq!(not_masked, placeholder_url);
    }

    #[test]
    fn test_global_config() {
        let config = get_config();
        assert!(!config.eth_rpc_url.is_empty());
        assert!(!config.eth_base_rpc_url.is_empty());
        assert!(!config.eth_op_rpc_url.is_empty());
        assert!(!config.farcaster_hub_url.is_empty());
    }
}
