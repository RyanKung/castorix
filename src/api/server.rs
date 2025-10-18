//! API server implementation

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::api::handlers::{contract, ens, hub};
use crate::api::routes;
use crate::core::client::FarcasterClient;
use crate::farcaster::contracts::FarcasterContractClient;
use crate::farcaster::contracts::ContractAddresses;

/// API server configuration
pub struct ApiServer {
    pub host: String,
    pub port: u16,
    pub hub_url: String,
    pub eth_rpc_url: Option<String>,
    pub eth_base_rpc_url: Option<String>,
    pub eth_op_rpc_url: Option<String>,
}

impl Default for ApiServer {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            hub_url: std::env::var("FARCASTER_HUB_URL")
                .unwrap_or_else(|_| "https://hub-api.neynar.com".to_string()),
            eth_rpc_url: std::env::var("ETH_RPC_URL").ok(),
            eth_base_rpc_url: std::env::var("ETH_BASE_RPC_URL").ok(),
            eth_op_rpc_url: std::env::var("ETH_OP_RPC_URL").ok(),
        }
    }
}

impl ApiServer {
    /// Create a new API server with custom configuration
    pub fn new(
        host: impl Into<String>,
        port: u16,
        hub_url: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            hub_url: hub_url.into(),
            ..Default::default()
        }
    }

    /// Start the API server
    pub async fn serve(self) -> Result<()> {
        info!("🚀 Starting Castorix REST API server");
        info!("   Host: {}", self.host);
        info!("   Port: {}", self.port);
        info!("   Hub URL: {}", self.hub_url);

        // Create Hub client
        let hub_client = Arc::new(FarcasterClient::new(self.hub_url.clone(), None));
        let hub_state = hub::HubState {
            client: hub_client,
        };

        // Create ENS state if RPC URL is available
        let ens_state = if self.eth_rpc_url.is_some() {
            info!("✅ ENS endpoints enabled");
            Some(ens::EnsState {
                eth_rpc_url: self.eth_rpc_url.clone().unwrap(),
                base_rpc_url: self.eth_base_rpc_url.clone(),
            })
        } else {
            info!("⚠️  ENS endpoints disabled (no ETH_RPC_URL)");
            None
        };

        // Create Contract state if Optimism RPC URL is available
        let contract_state = if let Some(op_rpc_url) = &self.eth_op_rpc_url {
            info!("✅ Contract endpoints enabled");
            let addresses = ContractAddresses::default();
            let client = FarcasterContractClient::new(op_rpc_url.to_string(), addresses)
                .context("Failed to create contract client")?;
            Some(contract::ContractState {
                client: Arc::new(client),
            })
        } else {
            info!("⚠️  Contract endpoints disabled (no ETH_OP_RPC_URL)");
            None
        };

        // Log available endpoints based on enabled features
        let has_ens = ens_state.is_some();
        let has_contract = contract_state.is_some();

        // Build router
        let app = routes::build_router(hub_state, ens_state, contract_state)
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .layer(TraceLayer::new_for_http());

        // Create server address
        let addr: SocketAddr = format!("{}:{}", self.host, self.port)
            .parse()
            .context("Invalid host:port combination")?;

        info!("🎯 API server listening on http://{}", addr);
        info!("📚 Available endpoints:");
        info!("   GET  /health - Health check");
        info!("   GET  /api/hub/info - Hub information");
        info!("   GET  /api/hub/users/:fid - User info");
        info!("   GET  /api/hub/users/:fid/profile - User profile");
        info!("   GET  /api/hub/users/:fid/stats - User stats");
        info!("   GET  /api/hub/users/:fid/followers - Followers");
        info!("   GET  /api/hub/users/:fid/following - Following");
        info!("   GET  /api/hub/users/:fid/casts - User casts");
        info!("   GET  /api/hub/spam/:fid - Spam check");

        if has_ens {
            info!("   GET  /api/ens/resolve/:domain - Resolve ENS");
            info!("   GET  /api/ens/verify/:domain/:address - Verify ownership");
        }

        if has_contract {
            info!("   GET  /api/contract/fid/price - FID price");
            info!("   GET  /api/contract/storage/price/:units - Storage price");
            info!("   GET  /api/contract/address/:address/fid - Check address FID");
        }

        // Start server
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .context("Failed to bind to address")?;

        axum::serve(listener, app)
            .await
            .context("Server error")?;

        Ok(())
    }
}

