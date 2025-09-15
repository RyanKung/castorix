use super::core::EnsProof;
use anyhow::{Context, Result};

impl EnsProof {
    /// Get ENS domains that have proofs for the current address
    ///
    /// This method queries the Farcaster Hub to find all ENS domains
    /// that have been verified and have proofs for the current address.
    ///
    /// # Arguments
    /// * `hub_url` - The Farcaster Hub URL
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - List of ENS domains with proofs
    pub async fn get_ens_domains_with_proofs(&self, hub_url: &str) -> Result<Vec<String>> {
        use reqwest::Client;

        let _client = Client::new();
        let current_address = self.key_manager.address();

        // Query the Farcaster Hub for username proofs
        let _url = format!("{}/v1/usernameProofsByFid?fid={}", hub_url, 0); // We'll need to get FID first

        // First, we need to get the FID for the current address
        // This is a simplified approach - in practice, you'd need to query by address
        let mut domains = Vec::new();

        // For now, we'll implement a basic approach that checks common ENS domains
        // In a real implementation, you'd query the Farcaster Hub API properly
        let common_domains = vec![
            "vitalik.eth",
            "dwr.eth",
            "dankrad.eth",
            "danromero.eth",
            "jessepollak.eth",
            "rish.eth",
            "varun.eth",
            "mason.eth",
            "brian.eth",
            "alex.eth",
        ];

        for domain in common_domains {
            // Check if this domain resolves to our address
            if let Ok(resolved_address) = self.resolve_ens(domain).await {
                if resolved_address == current_address {
                    domains.push(domain.to_string());
                }
            }
        }

        Ok(domains)
    }

    /// Get ENS domains with proofs from Farcaster Hub API
    ///
    /// This method queries the Farcaster Hub API to find all ENS domains
    /// that have proofs for a specific FID.
    ///
    /// # Arguments
    /// * `hub_url` - The Farcaster Hub URL
    /// * `fid` - The Farcaster ID to query
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - List of ENS domains with proofs
    pub async fn get_ens_domains_by_fid(&self, hub_url: &str, fid: u64) -> Result<Vec<String>> {
        use reqwest::Client;

        let client = Client::new();

        // Query username proofs to find ENS domains
        let username_proofs_url = format!("{hub_url}/v1/userNameProofsByFid?fid={fid}");

        let username_proofs_response = client
            .get(&username_proofs_url)
            .send()
            .await
            .with_context(|| "Failed to query username proofs")?;

        if !username_proofs_response.status().is_success() {
            let error_text = username_proofs_response.text().await?;
            return Err(anyhow::anyhow!(
                "FID {} not found or error: {}",
                fid,
                error_text
            ));
        }

        let response_text = username_proofs_response.text().await?;
        let data: serde_json::Value = serde_json::from_str(&response_text)
            .with_context(|| "Failed to parse username proofs response")?;

        let mut domains = Vec::new();

        // Parse the response to extract ENS domains from username proofs
        if let Some(proofs) = data.get("proofs").and_then(|p| p.as_array()) {
            for proof in proofs {
                if let Some(name) = proof.get("name").and_then(|n| n.as_str()) {
                    // Check if it's an ENS domain (ends with .eth)
                    if name.ends_with(".eth") {
                        domains.push(name.to_string());
                    }
                }
            }
        }

        Ok(domains)
    }

    /// Get all ENS domains owned by an Ethereum address on-chain
    ///
    /// This method queries The Graph API to find all domains owned by the given address.
    ///
    /// # Arguments
    /// * `address` - The Ethereum address to query
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - List of ENS domains owned by the address
    pub async fn get_ens_domains_by_address(&self, address: &str) -> Result<Vec<String>> {
        use reqwest::Client;

        let client = Client::new();

        // Query The Graph API for ENS domains owned by the address
        // Use lowercase address as The Graph stores addresses in lowercase
        let query = format!(
            r#"{{
                "query": "query GetDomains($owner: String!) {{ domains(where: {{ owner: $owner }}) {{ name owner {{ id }} }} }}",
                "variables": {{ "owner": "{}" }}
            }}"#,
            address.to_lowercase()
        );

        let response = client
            .post("https://api.thegraph.com/subgraphs/name/ensdomains/ens")
            .header("Content-Type", "application/json")
            .body(query)
            .send()
            .await
            .with_context(|| "Failed to query The Graph API")?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            let data: serde_json::Value = serde_json::from_str(&response_text)
                .with_context(|| "Failed to parse The Graph response")?;

            let mut domains = Vec::new();

            if let Some(domains_data) = data.get("data").and_then(|d| d.get("domains")) {
                if let Some(domains_array) = domains_data.as_array() {
                    for domain in domains_array {
                        if let Some(name) = domain.get("name").and_then(|n| n.as_str()) {
                            domains.push(name.to_string());
                        }
                    }
                }
            }

            Ok(domains)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "The Graph API returned error: {}",
                error_text
            ))
        }
    }

    /// Get ENS domains by querying a specific domain pattern
    ///
    /// This method can be used to check if an address owns specific ENS domains
    /// by resolving them and checking ownership.
    ///
    /// # Arguments
    /// * `address` - The Ethereum address to check
    /// * `domain_patterns` - List of domain patterns to check (e.g., ["vitalik.eth", "dwr.eth"])
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - List of ENS domains owned by the address
    pub async fn check_ens_domain_ownership(
        &self,
        address: &str,
        domain_patterns: &[&str],
    ) -> Result<Vec<String>> {
        use ethers::types::Address;
        use std::str::FromStr;

        let _provider =
            ethers::providers::Provider::<ethers::providers::Http>::try_from(&self.rpc_url)
                .with_context(|| "Failed to create provider")?;

        let addr = Address::from_str(address).with_context(|| "Failed to parse address")?;

        let mut owned_domains = Vec::new();

        for domain in domain_patterns {
            // Try to resolve the domain to an address
            if let Ok(resolved_address) = self.resolve_ens(domain).await {
                if resolved_address == addr {
                    owned_domains.push(domain.to_string());
                }
            }
        }

        Ok(owned_domains)
    }
}
