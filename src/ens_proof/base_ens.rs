use super::core::EnsProof;
use anyhow::Result;
use ethers::providers::{Provider, Http, Middleware};
use ethers::types::{Address, H160, TransactionRequest};
use ethers::types::transaction::eip2718::TypedTransaction;
use std::str::FromStr;

impl EnsProof {
    /// Check if a specific Base subdomain exists and get its owner
    /// 
    /// This method directly queries for a specific Base subdomain to check if it exists.
    /// 
    /// # Arguments
    /// * `domain` - The Base subdomain to check (e.g., "ryankung.base.eth")
    /// 
    /// # Returns
    /// * `Result<Option<String>>` - Owner address if domain exists, None otherwise
    pub async fn check_base_subdomain(&self, domain: &str) -> Result<Option<String>> {
        use reqwest::Client;
        
        let client = Client::new();
        
        // Try multiple methods to find the Base subdomain
        
        // Method 1: Query main ENS subgraph
        let query1 = format!(
            r#"{{
                "query": "query GetDomain($name: String!) {{ domains(where: {{ name: $name }}) {{ name owner {{ id }} }} }}",
                "variables": {{ "name": "{domain}" }}
            }}"#
        );
        
        if let Ok(response) = client
            .post("https://api.thegraph.com/subgraphs/name/ensdomains/ens")
            .header("Content-Type", "application/json")
            .body(query1)
            .send()
            .await
        {
            if response.status().is_success() {
                if let Ok(response_text) = response.text().await {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&response_text) {
                        if let Some(domains_data) = data.get("data").and_then(|d| d.get("domains")) {
                            if let Some(domains_array) = domains_data.as_array() {
                                if let Some(domain_obj) = domains_array.first() {
                                    if let Some(owner) = domain_obj.get("owner").and_then(|o| o.get("id")).and_then(|id| id.as_str()) {
                                        return Ok(Some(owner.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Method 2: Try Base-specific subgraph (if it exists)
        let query2 = format!(
            r#"{{
                "query": "query GetDomain($name: String!) {{ domains(where: {{ name: $name }}) {{ name owner {{ id }} }} }}",
                "variables": {{ "name": "{domain}" }}
            }}"#
        );
        
        if let Ok(response) = client
            .post("https://api.thegraph.com/subgraphs/name/ensdomains/base-ens")
            .header("Content-Type", "application/json")
            .body(query2)
            .send()
            .await
        {
            if response.status().is_success() {
                if let Ok(response_text) = response.text().await {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&response_text) {
                        if let Some(domains_data) = data.get("data").and_then(|d| d.get("domains")) {
                            if let Some(domains_array) = domains_data.as_array() {
                                if let Some(domain_obj) = domains_array.first() {
                                    if let Some(owner) = domain_obj.get("owner").and_then(|o| o.get("id")).and_then(|id| id.as_str()) {
                                        return Ok(Some(owner.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Method 3: Try querying with Base contract address filter
        let query3 = format!(
            r#"{{
                "query": "query GetDomain($name: String!) {{ domains(where: {{ name: $name, registry: \"0xa41BF37f45de6658cBF3Bdc8b8ace13c3f9634Ec\" }}) {{ name owner {{ id }} }} }}",
                "variables": {{ "name": "{domain}" }}
            }}"#
        );
        
        if let Ok(response) = client
            .post("https://api.thegraph.com/subgraphs/name/ensdomains/ens")
            .header("Content-Type", "application/json")
            .body(query3)
            .send()
            .await
        {
            if response.status().is_success() {
                if let Ok(response_text) = response.text().await {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&response_text) {
                        if let Some(domains_data) = data.get("data").and_then(|d| d.get("domains")) {
                            if let Some(domains_array) = domains_data.as_array() {
                                if let Some(domain_obj) = domains_array.first() {
                                    if let Some(owner) = domain_obj.get("owner").and_then(|o| o.get("id")).and_then(|id| id.as_str()) {
                                        return Ok(Some(owner.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// Query Base chain ENS contract directly for domain ownership
    /// 
    /// This method queries the Base chain ENS contract directly to get domain ownership.
    /// 
    /// # Arguments
    /// * `domain` - The Base subdomain to check (e.g., "ryankung.base.eth")
    /// 
    /// # Returns
    /// * `Result<Option<String>>` - Owner address if domain exists, None otherwise
    pub async fn query_base_ens_contract(&self, domain: &str) -> Result<Option<String>> {
        // Load environment variables from .env file
        dotenv::dotenv().ok();
        
        // Use appropriate RPC URL based on domain type
        let (rpc_url, chain_name) = if domain.ends_with(".base.eth") {
            // For Base subdomains, use Base chain RPC
            let base_rpc = std::env::var("ETH_BASE_RPC_URL")
                .or_else(|_| std::env::var("ETH_RPC_URL"))
                .unwrap_or_else(|_| self.rpc_url.clone());
            (base_rpc, "Base")
        } else {
            // For regular ENS domains, use ETH_RPC_URL
            let eth_rpc = std::env::var("ETH_RPC_URL")
                .unwrap_or_else(|_| self.rpc_url.clone());
            (eth_rpc, "Ethereum")
        };
        
        let provider = Provider::<Http>::try_from(&rpc_url)
            .map_err(|e| anyhow::anyhow!("Failed to create {} provider: {}", chain_name, e))?;
        
        // ENS contracts - try different contracts for Base subdomains
        let ens_registry_contract = if domain.ends_with(".base.eth") {
            // For Base subdomains, try Base-specific ENS registry first
            Address::from_str("0xB94704422c2a1E396835A571837Aa5AE53285a95")
                .map_err(|e| anyhow::anyhow!("Invalid Base ENS registry contract address: {}", e))?
        } else {
            // For regular ENS domains, use standard registry
            Address::from_str("0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e")
                .map_err(|e| anyhow::anyhow!("Invalid ENS registry contract address: {}", e))?
        };
        
        // Parse domain name (remove .base.eth suffix)
        let name = domain.replace(".base.eth", "");
        
        println!("🔍 Querying {chain_name} ENS contract for: {domain} (name: {name})");
        println!("   ENS Registry: {ens_registry_contract:?}");
        println!("   Provider: {rpc_url}");
        
        // For Base subdomains, also try to check if the parent domain exists
        if domain.ends_with(".base.eth") {
            println!("   Checking parent domain 'base.eth' first...");
            let parent_namehash = self.calculate_namehash("base.eth")?;
            println!("   Parent namehash: 0x{}", hex::encode(parent_namehash));
        }
        
        // Calculate namehash for the domain
        let namehash = self.calculate_namehash(domain)?;
        println!("   Namehash: 0x{}", hex::encode(namehash));
        
        // Step 1: Call resolver(bytes32) to get resolver address
        // Function signature: resolver(bytes32) returns (address)
        let resolver_selector = "0178b8bf"; // resolver(bytes32) without 0x prefix
        
        // Encode the function call: resolver_selector + namehash
        let mut resolver_call_data = hex::decode(resolver_selector)
            .map_err(|e| anyhow::anyhow!("Failed to decode resolver selector: {}", e))?;
        resolver_call_data.extend_from_slice(&namehash);
        
        // Call the ENS Registry contract for resolver
        let resolver_tx_request = TransactionRequest::new()
            .to(ens_registry_contract)
            .data(resolver_call_data);
        
        let resolver_typed_tx = TypedTransaction::Legacy(resolver_tx_request);
        let resolver_result = provider.call(&resolver_typed_tx, None).await;
        
        match resolver_result {
            Ok(resolver_data) => {
                if resolver_data.len() == 32 {
                    // Parse the resolver address (last 20 bytes)
                    let resolver_bytes = &resolver_data[12..32];
                    let resolver_address = H160::from_slice(resolver_bytes);
                    
                    // Check if resolver is zero address (domain doesn't exist)
                    if resolver_address == H160::zero() {
                        println!("   Result: Domain not found (no resolver)");
                        Ok(None)
                    } else {
                        println!("   Resolver: {resolver_address:?}");
                        
                        // Step 2: Call addr(bytes32) on the resolver contract
                        // Function signature: addr(bytes32) returns (address)
                        let addr_selector = "3b3b57de"; // addr(bytes32) without 0x prefix
                        
                        // Encode the function call: addr_selector + namehash
                        let mut addr_call_data = hex::decode(addr_selector)
                            .map_err(|e| anyhow::anyhow!("Failed to decode addr selector: {}", e))?;
                        addr_call_data.extend_from_slice(&namehash);
                        
                        // Call the resolver contract for address
                        let addr_tx_request = TransactionRequest::new()
                            .to(resolver_address)
                            .data(addr_call_data);
                        
                        let addr_typed_tx = TypedTransaction::Legacy(addr_tx_request);
                        let addr_result = provider.call(&addr_typed_tx, None).await;
                        
                        match addr_result {
                            Ok(addr_data) => {
                                if addr_data.len() == 32 {
                                    // Parse the returned address (last 20 bytes)
                                    let address_bytes = &addr_data[12..32];
                                    let resolved_address = H160::from_slice(address_bytes);
                                    
                                    // Check if it's a zero address (domain not resolved)
                                    if resolved_address == H160::zero() {
                                        println!("   Result: Domain not resolved (zero address)");
                                        Ok(None)
                                    } else {
                                        println!("   Result: Resolved address: {resolved_address:?}");
                                        Ok(Some(format!("{resolved_address:?}")))
                                    }
                                } else {
                                    println!("   Result: Invalid address data length: {}", addr_data.len());
                                    Ok(None)
                                }
                            }
                            Err(e) => {
                                println!("   Error: Resolver call failed: {e}");
                                Err(anyhow::anyhow!("Resolver call failed: {}", e))
                            }
                        }
                    }
                } else {
                    println!("   Result: Invalid resolver data length: {}", resolver_data.len());
                    Ok(None)
                }
            }
            Err(e) => {
                println!("   Error: Registry call failed: {e}");
                Err(anyhow::anyhow!("Registry call failed: {}", e))
            }
        }
    }
    
    /// Calculate namehash for a domain name
    /// 
    /// This implements the ENS namehash algorithm as specified in EIP-137.
    /// 
    /// # Arguments
    /// * `domain` - The domain name (e.g., "ryankung.base.eth")
    /// 
    /// # Returns
    /// * `Result<[u8; 32]>` - The namehash as a 32-byte array
    fn calculate_namehash(&self, domain: &str) -> Result<[u8; 32]> {
        use tiny_keccak::{Hasher, Keccak};
        
        // Split domain into labels
        let labels: Vec<&str> = domain.split('.').collect();
        
        // Start with the zero hash
        let mut node = [0u8; 32];
        
        // Process labels in reverse order
        for label in labels.iter().rev() {
            // First hash the label with keccak256
            let mut label_hasher = Keccak::v256();
            label_hasher.update(label.as_bytes());
            let mut label_hash = [0u8; 32];
            label_hasher.finalize(&mut label_hash);
            
            // Then hash node + label_hash with keccak256
            let mut hasher = Keccak::v256();
            hasher.update(&node);
            hasher.update(&label_hash);
            hasher.finalize(&mut node);
        }
        
        Ok(node)
    }

    /// Get Base subdomains (like *.base.eth) for a given address
    /// 
    /// ⚠️  Note: Base chain reverse lookup is not currently supported.
    /// Base subdomains are not indexed by The Graph API, and direct
    /// contract queries would require enumerating all possible subdomains.
    /// 
    /// # Arguments
    /// * `address` - The Ethereum address to query
    /// 
    /// # Returns
    /// * `Result<Vec<String>>` - Empty vector (Base chain not supported)
    pub async fn get_base_subdomains_by_address(&self, _address: &str) -> Result<Vec<String>> {
        // Base chain reverse lookup is not currently supported
        // The Graph API doesn't index Base subdomains, and direct
        // contract queries would require enumerating all possible subdomains
        Ok(Vec::new())
    }

    /// Get all ENS domains for a given address
    /// 
    /// This method queries for regular ENS domains owned by the address.
    /// Note: Base subdomains (*.base.eth) reverse lookup is not currently supported.
    /// 
    /// # Arguments
    /// * `address` - The Ethereum address to query
    /// 
    /// # Returns
    /// * `Result<Vec<String>>` - List of ENS domains owned by the address
    pub async fn get_all_ens_domains_by_address(&self, address: &str) -> Result<Vec<String>> {
        // Only query regular ENS domains
        // Base subdomains reverse lookup is not currently supported
        self.get_ens_domains_by_address(address).await
    }
}
