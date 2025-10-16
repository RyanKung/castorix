//! Farcaster Hub query tools for MCP

use std::sync::Arc;

use async_trait::async_trait;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

use crate::core::client::FarcasterClient;
use crate::mcp::error::McpError;
use crate::mcp::error::Result;
use crate::mcp::tools::base::McpTool;
use crate::mcp::types::InputSchema;
use crate::mcp::types::Tool;
use crate::mcp::utils::SpamChecker;
use crate::mcp::utils::SpamStatus;

// Lazy-load spam checker once
lazy_static! {
    static ref SPAM_CHECKER: Option<SpamChecker> = {
        match SpamChecker::load() {
            Ok(checker) => Some(checker),
            Err(e) => {
                eprintln!("Warning: Failed to load spam checker: {}", e);
                None
            }
        }
    };
}

/// Hub client wrapper for tools
pub struct HubContext {
    pub client: Arc<FarcasterClient>,
}

impl HubContext {
    pub fn new(hub_url: String) -> Self {
        Self {
            client: Arc::new(FarcasterClient::new(hub_url, None)),
        }
    }
}

// ============================================================================
// 1. hub_get_user - Get user information by FID
// ============================================================================

pub struct HubGetUserTool {
    context: Arc<HubContext>,
}

impl HubGetUserTool {
    pub fn new(context: Arc<HubContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetUserArgs {
    fid: u64,
}

#[async_trait]
impl McpTool for HubGetUserTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_user".to_string(),
            description: "Get Farcaster user information by FID. Returns username, display name, bio, avatar, and follower counts.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fid": {
                        "type": "number",
                        "description": "The Farcaster ID (FID) of the user to query"
                    }
                }),
                required: vec!["fid".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetUserArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid FID: {}", e)))?;

        let user_data = self
            .context
            .client
            .get_user(args.fid)
            .await
            .map_err(|e| McpError::HubConnectionFailed(format!("Failed to get user: {}", e)))?;

        serde_json::to_value(user_data).map_err(McpError::SerializationError)
    }
}

// ============================================================================
// 2. hub_get_profile - Get detailed user profile
// ============================================================================

pub struct HubGetProfileTool {
    context: Arc<HubContext>,
}

impl HubGetProfileTool {
    pub fn new(context: Arc<HubContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetProfileArgs {
    fid: u64,
    #[serde(default)]
    all: bool,
}

#[async_trait]
impl McpTool for HubGetProfileTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_profile".to_string(),
            description: "Get detailed Farcaster user profile by FID. Use 'all: true' to get complete profile information including all metadata.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fid": {
                        "type": "number",
                        "description": "The Farcaster ID (FID) of the user to query"
                    },
                    "all": {
                        "type": "boolean",
                        "description": "Whether to return all profile information (default: false)",
                        "default": false
                    }
                }),
                required: vec!["fid".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetProfileArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        // Get basic user data
        let user_data =
            self.context.client.get_user(args.fid).await.map_err(|e| {
                McpError::HubConnectionFailed(format!("Failed to get profile: {}", e))
            })?;

        let mut profile = serde_json::to_value(user_data).map_err(McpError::SerializationError)?;

        if args.all {
            // Fetch additional profile data when 'all' is true
            if let Some(obj) = profile.as_object_mut() {
                // Get verified Ethereum addresses
                if let Ok(eth_addresses) = self.context.client.get_eth_addresses(args.fid).await {
                    obj.insert("verified_addresses".to_string(), json!(eth_addresses));
                }

                // Get custody address
                if let Ok(custody_address) = self.context.client.get_custody_address(args.fid).await
                {
                    obj.insert("custody_address".to_string(), json!(custody_address));
                }

                obj.insert("complete_profile".to_string(), json!(true));
            }
        }

        Ok(profile)
    }
}

// ============================================================================
// 3. hub_get_stats - Get user statistics
// ============================================================================

pub struct HubGetStatsTool {
    context: Arc<HubContext>,
}

impl HubGetStatsTool {
    pub fn new(context: Arc<HubContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetStatsArgs {
    fid: u64,
}

#[async_trait]
impl McpTool for HubGetStatsTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_stats".to_string(),
            description: "Get user statistics for a Farcaster ID. Returns follower count, following count, and storage usage information.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fid": {
                        "type": "number",
                        "description": "The Farcaster ID (FID) to get statistics for"
                    }
                }),
                required: vec!["fid".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetStatsArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid FID: {}", e)))?;

        // Get user data which includes follower/following counts
        let user_data =
            self.context.client.get_user(args.fid).await.map_err(|e| {
                McpError::HubConnectionFailed(format!("Failed to get stats: {}", e))
            })?;

        // Convert to Value and extract relevant stats
        let user_value = serde_json::to_value(&user_data).map_err(McpError::SerializationError)?;

        // Build stats response
        let stats = json!({
            "fid": args.fid,
            "follower_count": user_value.get("follower_count").unwrap_or(&json!(0)),
            "following_count": user_value.get("following_count").unwrap_or(&json!(0)),
        });

        Ok(stats)
    }
}

// ============================================================================
// 4. hub_get_followers - Get followers list
// ============================================================================

pub struct HubGetFollowersTool {
    context: Arc<HubContext>,
}

impl HubGetFollowersTool {
    pub fn new(context: Arc<HubContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetFollowersArgs {
    fid: u64,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 {
    1000
}

#[async_trait]
impl McpTool for HubGetFollowersTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_followers".to_string(),
            description: "Get the list of followers for a Farcaster ID. Returns an array of FIDs that follow this user.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fid": {
                        "type": "number",
                        "description": "The Farcaster ID (FID) to get followers for"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of followers to return (default: 1000, 0 for unlimited)",
                        "default": 1000
                    }
                }),
                required: vec!["fid".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetFollowersArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        let followers = self
            .context
            .client
            .get_followers(args.fid, args.limit)
            .await
            .map_err(|e| {
                McpError::HubConnectionFailed(format!("Failed to get followers: {}", e))
            })?;

        Ok(json!({
            "fid": args.fid,
            "followers": followers,
            "count": followers.len()
        }))
    }
}

// ============================================================================
// 5. hub_get_following - Get following list
// ============================================================================

pub struct HubGetFollowingTool {
    context: Arc<HubContext>,
}

impl HubGetFollowingTool {
    pub fn new(context: Arc<HubContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetFollowingArgs {
    fid: u64,
    #[serde(default = "default_limit")]
    limit: u32,
}

#[async_trait]
impl McpTool for HubGetFollowingTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_following".to_string(),
            description: "Get the list of users that a Farcaster ID follows. Returns an array of FIDs that this user follows.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fid": {
                        "type": "number",
                        "description": "The Farcaster ID (FID) to get following list for"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of following to return (default: 1000, 0 for unlimited)",
                        "default": 1000
                    }
                }),
                required: vec!["fid".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetFollowingArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        let following = self
            .context
            .client
            .get_following(args.fid, args.limit)
            .await
            .map_err(|e| {
                McpError::HubConnectionFailed(format!("Failed to get following: {}", e))
            })?;

        Ok(json!({
            "fid": args.fid,
            "following": following,
            "count": following.len()
        }))
    }
}

// ============================================================================
// 6. hub_get_eth_addresses - Get Ethereum addresses
// ============================================================================

pub struct HubGetEthAddressesTool {
    context: Arc<HubContext>,
}

impl HubGetEthAddressesTool {
    pub fn new(context: Arc<HubContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetEthAddressesArgs {
    fid: u64,
}

#[async_trait]
impl McpTool for HubGetEthAddressesTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_eth_addresses".to_string(),
            description: "Get verified Ethereum addresses for a Farcaster ID. Returns an array of Ethereum addresses that have been verified by this FID.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fid": {
                        "type": "number",
                        "description": "The Farcaster ID (FID) to get Ethereum addresses for"
                    }
                }),
                required: vec!["fid".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetEthAddressesArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid FID: {}", e)))?;

        let addresses = self
            .context
            .client
            .get_eth_addresses(args.fid)
            .await
            .map_err(|e| {
                McpError::HubConnectionFailed(format!("Failed to get Ethereum addresses: {}", e))
            })?;

        Ok(json!({
            "fid": args.fid,
            "addresses": addresses,
            "count": addresses.len()
        }))
    }
}

// ============================================================================
// 7. hub_get_custody_address - Get custody address
// ============================================================================

pub struct HubGetCustodyAddressTool {
    context: Arc<HubContext>,
}

impl HubGetCustodyAddressTool {
    pub fn new(context: Arc<HubContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetCustodyAddressArgs {
    fid: u64,
}

#[async_trait]
impl McpTool for HubGetCustodyAddressTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_custody_address".to_string(),
            description: "Get the custody address for a Farcaster ID. The custody address is the Ethereum address that controls this FID and can perform administrative actions.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fid": {
                        "type": "number",
                        "description": "The Farcaster ID (FID) to get custody address for"
                    }
                }),
                required: vec!["fid".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetCustodyAddressArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid FID: {}", e)))?;

        let custody_address = self
            .context
            .client
            .get_custody_address(args.fid)
            .await
            .map_err(|e| {
                McpError::HubConnectionFailed(format!("Failed to get custody address: {}", e))
            })?;

        Ok(json!({
            "fid": args.fid,
            "custody_address": custody_address
        }))
    }
}

// ============================================================================
// 8. hub_get_info - Get Hub information
// ============================================================================

pub struct HubGetInfoTool {
    context: Arc<HubContext>,
}

impl HubGetInfoTool {
    pub fn new(context: Arc<HubContext>) -> Self {
        Self { context }
    }
}

#[async_trait]
impl McpTool for HubGetInfoTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_info".to_string(),
            description: "Get Farcaster Hub information and synchronization status. Returns version info, sync status, and shard information.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({}),
                required: vec![],
            },
        }
    }

    async fn execute(&self, _arguments: Value) -> Result<Value> {
        let hub_info =
            self.context.client.get_hub_info().await.map_err(|e| {
                McpError::HubConnectionFailed(format!("Failed to get Hub info: {}", e))
            })?;

        Ok(hub_info)
    }
}

// ============================================================================
// 9. hub_get_ens_domains - Get ENS domains with proofs
// ============================================================================

pub struct HubGetEnsDomainsTool {
    context: Arc<HubContext>,
}

impl HubGetEnsDomainsTool {
    pub fn new(context: Arc<HubContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetEnsDomainsArgs {
    fid: u64,
}

#[async_trait]
impl McpTool for HubGetEnsDomainsTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_ens_domains".to_string(),
            description: "Get ENS domains with username proofs for a Farcaster ID. Returns all ENS domains (including Basenames) that have been verified for this FID.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fid": {
                        "type": "number",
                        "description": "The Farcaster ID (FID) to get ENS domains for"
                    }
                }),
                required: vec!["fid".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetEnsDomainsArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid FID: {}", e)))?;

        // Query username proofs from Hub API
        let url = format!(
            "{}/v1/userNameProofsByFid?fid={}",
            self.context.client.hub_url(),
            args.fid
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| McpError::HubConnectionFailed(format!("Failed to query Hub: {}", e)))?;

        let status = response.status();
        let response_text = response.text().await.map_err(|e| {
            McpError::HubConnectionFailed(format!("Failed to read response: {}", e))
        })?;

        if !status.is_success() {
            return Err(McpError::HubConnectionFailed(format!(
                "Hub returned error {}: {}",
                status, response_text
            )));
        }

        let data: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
            McpError::HubConnectionFailed(format!("Failed to parse response: {}", e))
        })?;

        // Extract domain names from proofs
        let mut domains = Vec::new();
        if let Some(proofs) = data.get("proofs").and_then(|p| p.as_array()) {
            for proof in proofs {
                if let Some(name) = proof.get("name").and_then(|n| n.as_str()) {
                    domains.push(name.to_string());
                }
            }
        }

        Ok(json!({
            "fid": args.fid,
            "domains": domains,
            "count": domains.len()
        }))
    }
}

// ============================================================================
// 10. hub_check_spam - Check spam status
// ============================================================================

pub struct HubCheckSpamTool;

impl HubCheckSpamTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HubCheckSpamTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct CheckSpamArgs {
    fids: Vec<u64>,
}

#[async_trait]
impl McpTool for HubCheckSpamTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_check_spam".to_string(),
            description: "Check if one or more FIDs are marked as spam in Warpcast's spam labels dataset. Returns spam status for each FID.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fids": {
                        "type": "array",
                        "items": {
                            "type": "number"
                        },
                        "description": "Array of Farcaster IDs (FIDs) to check for spam status"
                    }
                }),
                required: vec!["fids".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: CheckSpamArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        let checker = SPAM_CHECKER
            .as_ref()
            .ok_or_else(|| McpError::InternalError("Spam checker not available".to_string()))?;

        let mut results = Vec::new();
        for fid in args.fids {
            let status = checker.get_status(fid);
            results.push(json!({
                "fid": fid,
                "is_spam": matches!(status, SpamStatus::Spam),
                "status": match status {
                    SpamStatus::Spam => "spam",
                    SpamStatus::NotSpam => "not_spam",
                    SpamStatus::Unknown => "unknown",
                }
            }));
        }

        Ok(json!({ "results": results }))
    }
}

// ============================================================================
// 11. hub_get_spam_stats - Get spam statistics
// ============================================================================

pub struct HubGetSpamStatsTool;

impl HubGetSpamStatsTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HubGetSpamStatsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl McpTool for HubGetSpamStatsTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_spam_stats".to_string(),
            description: "Get comprehensive spam statistics from Warpcast's spam labels dataset. Returns total labels, spam count, non-spam count, and percentages.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({}),
                required: vec![],
            },
        }
    }

    async fn execute(&self, _arguments: Value) -> Result<Value> {
        let checker = SPAM_CHECKER
            .as_ref()
            .ok_or_else(|| McpError::InternalError("Spam checker not available".to_string()))?;

        let stats = checker.get_stats();

        Ok(json!({
            "total_labels": stats.total_labels,
            "unique_fids": stats.unique_fids,
            "spam_count": stats.spam_count,
            "non_spam_count": stats.non_spam_count,
            "spam_percentage": format!("{:.2}%", stats.spam_percentage),
        }))
    }
}

// ============================================================================
// 12. hub_get_casts - Get casts by FID
// ============================================================================

pub struct HubGetCastsTool {
    context: Arc<HubContext>,
}

impl HubGetCastsTool {
    pub fn new(context: Arc<HubContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetCastsArgs {
    fid: u64,
    #[serde(default = "default_cast_limit")]
    limit: u32,
}

fn default_cast_limit() -> u32 {
    100
}

#[async_trait]
impl McpTool for HubGetCastsTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "hub_get_casts".to_string(),
            description: "Get casts (posts/messages) published by a Farcaster ID. Returns recent casts with text, embeds, mentions, reactions, and metadata.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fid": {
                        "type": "number",
                        "description": "The Farcaster ID (FID) to get casts for"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of casts to return (default: 100, 0 for unlimited)",
                        "default": 100
                    }
                }),
                required: vec!["fid".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetCastsArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        let casts = self
            .context
            .client
            .get_casts_by_fid(args.fid, args.limit)
            .await
            .map_err(|e| McpError::HubConnectionFailed(format!("Failed to get casts: {}", e)))?;

        Ok(json!({
            "fid": args.fid,
            "casts": casts,
            "count": casts.len()
        }))
    }
}

/// Create all Hub tools
pub fn create_hub_tools(context: Arc<HubContext>) -> Vec<Box<dyn McpTool>> {
    vec![
        // Phase 1: Core tools
        Box::new(HubGetUserTool::new(context.clone())),
        Box::new(HubGetProfileTool::new(context.clone())),
        Box::new(HubGetStatsTool::new(context.clone())),
        Box::new(HubGetFollowersTool::new(context.clone())),
        Box::new(HubGetFollowingTool::new(context.clone())),
        // Phase 2: Extended tools
        Box::new(HubGetEthAddressesTool::new(context.clone())),
        Box::new(HubGetCustodyAddressTool::new(context.clone())),
        Box::new(HubGetInfoTool::new(context.clone())),
        Box::new(HubGetEnsDomainsTool::new(context.clone())),
        Box::new(HubCheckSpamTool::new()),
        Box::new(HubGetSpamStatsTool::new()),
        // Cast queries
        Box::new(HubGetCastsTool::new(context)),
    ]
}
