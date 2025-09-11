use crate::{key_manager::KeyManager, username_proof::{UserNameProof, UserNameType}, message::{Message, MessageData, MessageType, FarcasterNetwork, HashScheme, SignatureScheme}};
use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use protobuf::Message as ProtobufMessage;
use ed25519_dalek::{SigningKey, Signer as Ed25519Signer};

/// Farcaster Hub client for submitting messages and proofs
pub struct FarcasterClient {
    client: Client,
    hub_url: String,
    key_manager: Option<KeyManager>,
}

/// Farcaster message structure (using protobuf Message)
pub type FarcasterMessage = Message;

/// Signer information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerInfo {
    /// The signer public key (Ed25519)
    pub key: String,
    /// The key type (1 for Ed25519)
    pub key_type: u32,
    /// The event type (SIGNER_EVENT_TYPE_ADD for active signers)
    pub event_type: String,
}

/// Username proof data for Farcaster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameProofData {
    pub timestamp: u64,
    pub name: String,
    pub owner: String,
    pub signature: String,
    pub fid: u64,
    #[serde(rename = "type")]
    pub proof_type: String,
}

/// Cast message data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastData {
    pub text: String,
    pub mentions: Vec<u64>,
    pub mentions_positions: Vec<u32>,
    pub embeds: Vec<serde_json::Value>,
    pub parent_cast_id: Option<CastId>,
}

/// Cast ID structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastId {
    pub fid: u64,
    pub hash: String,
}

/// Hub response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

impl FarcasterClient {
    /// Create a new Farcaster client
    /// 
    /// # Arguments
    /// * `hub_url` - The Farcaster Hub URL
    /// * `key_manager` - Optional key manager instance (required for write operations)
    /// 
    /// # Returns
    /// * `Self` - The FarcasterClient instance
    pub fn new(hub_url: String, key_manager: Option<KeyManager>) -> Self {
        Self {
            client: Client::new(),
            hub_url,
            key_manager,
        }
    }

    /// Create a new Farcaster client with key manager
    /// 
    /// # Arguments
    /// * `hub_url` - The Farcaster Hub URL
    /// * `key_manager` - The key manager instance
    /// 
    /// # Returns
    /// * `Self` - The FarcasterClient instance
    pub fn with_key_manager(hub_url: String, key_manager: KeyManager) -> Self {
        Self::new(hub_url, Some(key_manager))
    }

    /// Create a new Farcaster client from environment variables
    /// 
    /// # Returns
    /// * `Result<Self>` - The FarcasterClient instance or an error
    pub fn from_env() -> Result<Self> {
        let key_manager = KeyManager::from_env("PRIVATE_KEY")?;
        let hub_url = std::env::var("FARCASTER_HUB_URL")
            .unwrap_or_else(|_| "https://hub-api.neynar.com".to_string());
        
        Ok(Self::with_key_manager(hub_url, key_manager))
    }

    /// Create a new Farcaster client without authentication (read-only operations)
    /// 
    /// # Arguments
    /// * `hub_url` - The Farcaster Hub URL
    /// 
    /// # Returns
    /// * `Self` - The FarcasterClient instance
    pub fn read_only(hub_url: String) -> Self {
        Self::new(hub_url, None)
    }

    /// Submit a username proof to Farcaster Hub using EIP-712 signature
    /// 
    /// # Arguments
    /// * `proof` - The username proof to submit
    /// 
    /// # Returns
    /// * `Result<HubResponse>` - The hub response or an error
    pub async fn submit_username_proof_with_eip712(&self, proof: &UserNameProof) -> Result<HubResponse> {
        if self.key_manager.is_none() {
            anyhow::bail!("Key manager required for EIP-712 signing");
        }

        // Get the correct Ed25519 public key for this FID from the Hub
        let fid = proof.get_fid();
        let ed25519_public_key_hex = self.get_ed25519_public_key_from_hub(fid).await?;
        
        // Remove 0x prefix if present and decode the public key
        let clean_key = if let Some(stripped) = ed25519_public_key_hex.strip_prefix("0x") {
            stripped
        } else {
            &ed25519_public_key_hex
        };
        
        let _ed25519_public_key_bytes = hex::decode(clean_key)
            .with_context(|| "Failed to decode Ed25519 public key from hex")?;

        // Create MessageData with username proof
        let mut message_data = MessageData::new();
        // Use Farcaster time (seconds since January 1, 2021)
        const FARCASTER_EPOCH: u64 = 1609459200; // January 1, 2021 UTC in seconds
        let current_timestamp = Utc::now().timestamp() as u64;
        let farcaster_timestamp = (current_timestamp - FARCASTER_EPOCH) as u32;

        message_data.set_field_type(MessageType::MESSAGE_TYPE_USERNAME_PROOF);
        message_data.set_fid(proof.get_fid());
        message_data.set_timestamp(farcaster_timestamp);
        message_data.set_network(FarcasterNetwork::FARCASTER_NETWORK_MAINNET);

        // Set the username proof in the body with current timestamp
        let mut username_proof = proof.clone();
        username_proof.set_timestamp(farcaster_timestamp as u64);
        // Set the correct username type for Base domains
        username_proof.set_field_type(UserNameType::USERNAME_TYPE_BASENAME);
        message_data.set_username_proof_body(username_proof);

        // Create the Message wrapper
        let mut message = Message::new();
        message.set_data(message_data);

        // Calculate hash of the MessageData (using first 20 bytes of blake3 hash like Snapchain)
        let message_data_bytes = message.get_data().write_to_bytes()?;
        let hash = blake3::hash(&message_data_bytes);
        let hash_20 = hash.as_bytes()[..20].to_vec();
        message.set_hash(hash_20.clone());
        message.set_hash_scheme(HashScheme::HASH_SCHEME_BLAKE3);
        message.set_signature_scheme(SignatureScheme::SIGNATURE_SCHEME_ED25519);

        // Sign using Ed25519 with the Ethereum private key converted to Ed25519
        let key_manager = self.key_manager.as_ref().unwrap();
        let wallet = key_manager.wallet();
        
        // Convert Ethereum private key to Ed25519 for Farcaster
        let private_key_bytes = wallet.signer().to_bytes();
        let ed25519_signing_key = SigningKey::from_bytes(&private_key_bytes[..32].try_into().unwrap());
        let signature = ed25519_signing_key.sign(&hash_20);
        message.set_signature(signature.to_bytes().to_vec());
        
        // For Ed25519 signature scheme, the signer should be the Ed25519 public key
        message.set_signer(ed25519_signing_key.verifying_key().to_bytes().to_vec());
        
        // Set data_bytes field (required by Farcaster Hub)
        // According to Snapchain docs, when dataBytes is set, data should be undefined
        message.set_data_bytes(message_data_bytes);
        message.clear_data(); // Clear the data field as per Snapchain requirements

        self.submit_message(&message).await
    }

    /// Submit a username proof to Farcaster Hub using Ed25519 key
    /// 
    /// # Arguments
    /// * `proof` - The username proof to submit
    /// * `ed25519_key_name` - Name of the Ed25519 key to use for signing
    /// 
    /// # Returns
    /// * `Result<HubResponse>` - The hub response or an error
    pub async fn submit_username_proof_with_ed25519(&self, proof: &UserNameProof, fid: u64) -> Result<HubResponse> {
        // Load encrypted Ed25519 key manager
        let keys_file = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::default_keys_file()?;
        let ed25519_manager = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::load_from_file(&keys_file)?;
        
        // Check if Ed25519 key exists for this FID
        if !ed25519_manager.has_key(fid) {
            anyhow::bail!("❌ No Ed25519 key found for FID: {}\n💡 Please generate or import an Ed25519 key for this FID first:\n   castorix hub key generate {}\n   castorix hub key import {}", fid, fid, fid);
        }
        
        // Prompt for password
        let password = crate::encrypted_ed25519_key_manager::prompt_password(&format!("Enter password for FID {fid}: "))?;
        
        // Get the Ed25519 signing key
        let signing_key = ed25519_manager.get_signing_key(fid, &password)?;
        
        // Create MessageData with username proof
        let mut message_data = MessageData::new();
        // Use Farcaster time (seconds since January 1, 2021)
        const FARCASTER_EPOCH: u64 = 1609459200; // January 1, 2021 UTC in seconds
        let current_timestamp = Utc::now().timestamp() as u64;
        let farcaster_timestamp = (current_timestamp - FARCASTER_EPOCH) as u32;
        
        message_data.set_field_type(MessageType::MESSAGE_TYPE_USERNAME_PROOF);
        message_data.set_fid(proof.get_fid());
        message_data.set_timestamp(farcaster_timestamp);
        message_data.set_network(FarcasterNetwork::FARCASTER_NETWORK_MAINNET);
        
        // Set the username proof in the body with current timestamp
        let mut username_proof = proof.clone();
        username_proof.set_timestamp(farcaster_timestamp as u64);
        // Set the correct username type for Base domains
        username_proof.set_field_type(UserNameType::USERNAME_TYPE_BASENAME);
        message_data.set_username_proof_body(username_proof);

        // Create the Message wrapper
        let mut message = Message::new();
        message.set_data(message_data);

        // Calculate hash of the MessageData (using first 20 bytes of blake3 hash like Snapchain)
        let message_data_bytes = message.get_data().write_to_bytes()?;
        let hash = blake3::hash(&message_data_bytes);
        let hash_20 = hash.as_bytes()[..20].to_vec();
        message.set_hash(hash_20.clone());
        message.set_hash_scheme(HashScheme::HASH_SCHEME_BLAKE3);
        message.set_signature_scheme(SignatureScheme::SIGNATURE_SCHEME_ED25519);

        // Sign the hash using the Ed25519 signing key
        let signature = signing_key.sign(&hash_20);
        message.set_signature(signature.to_bytes().to_vec());
        message.set_signer(signing_key.verifying_key().to_bytes().to_vec());
        
        // Set data_bytes field (required by Farcaster Hub)
        // According to Snapchain docs, when dataBytes is set, data should be undefined
        message.set_data_bytes(message_data_bytes);
        message.clear_data(); // Clear the data field as per Snapchain requirements

        self.submit_message(&message).await
    }

    /// Submit a username proof to Farcaster Hub (legacy method using Ethereum key)
    /// 
    /// # Arguments
    /// * `proof` - The username proof to submit
    /// 
    /// # Returns
    /// * `Result<HubResponse>` - The hub response or an error
    pub async fn submit_username_proof(&self, proof: &UserNameProof) -> Result<HubResponse> {
        if self.key_manager.is_none() {
            return Err(anyhow::anyhow!("Key manager required for submitting proofs"));
        }
        
        // Create MessageData with username proof
        let mut message_data = MessageData::new();
        // Use Farcaster time (seconds since January 1, 2021)
        const FARCASTER_EPOCH: u64 = 1609459200; // January 1, 2021 UTC in seconds
        let current_timestamp = Utc::now().timestamp() as u64;
        let farcaster_timestamp = (current_timestamp - FARCASTER_EPOCH) as u32;
        
        message_data.set_field_type(MessageType::MESSAGE_TYPE_USERNAME_PROOF);
        message_data.set_fid(proof.get_fid());
        message_data.set_timestamp(farcaster_timestamp);
        message_data.set_network(FarcasterNetwork::FARCASTER_NETWORK_MAINNET);
        
        // Set the username proof in the body with current timestamp
        let mut username_proof = proof.clone();
        username_proof.set_timestamp(farcaster_timestamp as u64);
        // Set the correct username type for Base domains
        username_proof.set_field_type(UserNameType::USERNAME_TYPE_BASENAME);
        message_data.set_username_proof_body(username_proof);

        // Create the Message wrapper
        let mut message = Message::new();
        message.set_data(message_data);
        
        // Calculate hash of the MessageData (using first 20 bytes of blake3 hash like Snapchain)
        let message_data_bytes = message.get_data().write_to_bytes()?;
        let hash = blake3::hash(&message_data_bytes);
        let hash_20 = hash.as_bytes()[..20].to_vec();
        message.set_hash(hash_20.clone());
        message.set_hash_scheme(HashScheme::HASH_SCHEME_BLAKE3);
        message.set_signature_scheme(SignatureScheme::SIGNATURE_SCHEME_ED25519);
        
        // Sign the hash using the private key
        let key_manager = self.key_manager.as_ref().ok_or_else(|| anyhow::anyhow!("Key manager required for signing"))?;
        let wallet = key_manager.wallet();
        
        // Convert Ethereum private key to Ed25519 for Farcaster
        // We need to use the same private key bytes for both
        let private_key_bytes = wallet.signer().to_bytes();
        let ed25519_signing_key = SigningKey::from_bytes(&private_key_bytes[..32].try_into().unwrap());
        let signature = ed25519_signing_key.sign(&hash_20);
        message.set_signature(signature.to_bytes().to_vec());
        message.set_signer(ed25519_signing_key.verifying_key().to_bytes().to_vec());
        
        // Set data_bytes field (required by Farcaster Hub)
        // According to Snapchain docs, when dataBytes is set, data should be undefined
        message.set_data_bytes(message_data_bytes);
        message.clear_data(); // Clear the data field as per Snapchain requirements

        self.submit_message(&message).await
    }


    /// Submit a message to Farcaster Hub
    /// 
    /// # Arguments
    /// * `message` - The message to submit
    /// 
    /// # Returns
    /// * `Result<HubResponse>` - The hub response or an error
    async fn submit_message(&self, message: &FarcasterMessage) -> Result<HubResponse> {
        let url = format!("{}/v1/submitMessage", self.hub_url);
        
        // Serialize the message to protobuf format
        let message_data = message.write_to_bytes()?;
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .body(message_data)
            .send()
            .await
            .with_context(|| "Failed to send request to Farcaster Hub")?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            let hub_response: HubResponse = serde_json::from_str(&response_text)
                .unwrap_or_else(|_| HubResponse {
                    success: true,
                    message: Some("Message submitted successfully".to_string()),
                    data: Some(serde_json::json!({ "raw_response": response_text })),
                });
            Ok(hub_response)
        } else {
            Err(anyhow::anyhow!(
                "Farcaster Hub returned error {}: {}",
                status,
                response_text
            ))
        }
    }

    /// Get user information from Farcaster Hub
    /// 
    /// # Arguments
    /// * `fid` - The Farcaster ID
    /// 
    /// # Returns
    /// * `Result<serde_json::Value>` - The user information or an error
    pub async fn get_user(&self, fid: u64) -> Result<serde_json::Value> {
        let url = format!("{}/v1/userDataByFid?fid={}", self.hub_url, fid);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to get user data from Farcaster Hub")?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&response_text)
                .with_context(|| "Failed to parse user data response")
        } else {
            Err(anyhow::anyhow!(
                "Farcaster Hub returned error {}: {}",
                status,
                response_text
            ))
        }
    }

    /// Get Ed25519 public key for a FID from local storage
    /// 
    /// # Arguments
    /// * `fid` - The Farcaster ID
    /// 
    /// # Returns
    /// * `Result<String>` - Ed25519 public key in hex format or an error
    pub async fn get_ed25519_public_key(&self, fid: u64) -> Result<String> {
        get_ed25519_public_key_for_fid(fid).await
    }

    /// Get Ed25519 public key for a FID from Farcaster Hub
    /// 
    /// # Arguments
    /// * `fid` - The Farcaster ID
    /// 
    /// # Returns
    /// * `Result<String>` - Ed25519 public key in hex format or an error
    pub async fn get_ed25519_public_key_from_hub(&self, fid: u64) -> Result<String> {
        // Use the Farcaster Hub API to get user data
        let url = format!("{}/v1/userDataByFid?fid={}", self.hub_url, fid);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to get user data from Farcaster Hub")?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            let data: serde_json::Value = serde_json::from_str(&response_text)
                .with_context(|| "Failed to parse user data response")?;
            
            // Look for Ed25519 public key in the response
            if let Some(messages) = data.get("messages").and_then(|m| m.as_array()) {
                // Get the signer from the first message (all messages should have the same signer)
                if let Some(first_message) = messages.first() {
                    if let Some(signer) = first_message.get("signer").and_then(|s| s.as_str()) {
                        return Ok(signer.to_string());
                    }
                }
            }
            
            anyhow::bail!("❌ No Ed25519 public key found for FID: {}\n💡 This FID may not have registered an Ed25519 signer", fid);
        } else {
            anyhow::bail!("❌ Failed to get user data from Farcaster Hub: {} - {}", status, response_text);
        }
    }

    /// Get custody address for a FID
    /// 
    /// # Arguments
    /// * `fid` - The Farcaster ID
    /// 
    /// # Returns
    /// * `Result<String>` - Custody address (Ethereum address) or an error
    pub async fn get_custody_address(&self, fid: u64) -> Result<String> {
        // Use the Farcaster Hub API to get onchain events for the FID
        let url = format!("{}/v1/onChainEventsByFid?fid={}&event_type=EVENT_TYPE_ID_REGISTER", self.hub_url, fid);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to get onchain events from Farcaster Hub")?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            let data: serde_json::Value = serde_json::from_str(&response_text)
                .with_context(|| "Failed to parse onchain events response")?;
            
            // Look for ID_REGISTER events in the response
            if let Some(events) = data.get("events").and_then(|e| e.as_array()) {
                for event in events {
                    if let Some(event_type) = event.get("type").and_then(|t| t.as_str()) {
                        if event_type == "EVENT_TYPE_ID_REGISTER" {
                            if let Some(id_register_body) = event.get("idRegisterEventBody") {
                                if let Some(to_address) = id_register_body.get("to").and_then(|a| a.as_str()) {
                                    return Ok(to_address.to_string());
                                }
                            }
                        }
                    }
                }
            }
            
            anyhow::bail!("❌ No custody address found for FID: {}\n💡 This FID may not be registered or the address may not be available", fid);
        } else {
            anyhow::bail!("❌ Failed to get onchain events from Farcaster Hub: {} - {}", status, response_text);
        }
    }

    /// Get Ethereum addresses bound to a FID
    /// 
    /// # Arguments
    /// * `fid` - The Farcaster ID
    /// 
    /// # Returns
    /// * `Result<Vec<String>>` - List of Ethereum addresses or an error
    pub async fn get_eth_addresses(&self, fid: u64) -> Result<Vec<String>> {
        // Use the correct Farcaster Hub API endpoint for verifications
        let url = format!("{}/v1/verificationsByFid?fid={}", self.hub_url, fid);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to get verification data from Farcaster Hub")?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            let data: serde_json::Value = serde_json::from_str(&response_text)
                .with_context(|| "Failed to parse verification data response")?;
            
            // Extract Ethereum addresses from the response
            let mut addresses = Vec::new();
            
            // Parse the Farcaster message format correctly
            if let Some(messages) = data.get("messages").and_then(|m| m.as_array()) {
                for message in messages {
                    // Check if this is a verification message
                    if let Some(data_obj) = message.get("data") {
                        if let Some(message_type) = data_obj.get("type").and_then(|t| t.as_str()) {
                            if message_type == "MESSAGE_TYPE_VERIFICATION_ADD_ETH_ADDRESS" {
                                // Extract the verification body
                                if let Some(verification_body) = data_obj.get("verificationAddAddressBody") {
                                    if let Some(protocol) = verification_body.get("protocol").and_then(|p| p.as_str()) {
                                        // Only include Ethereum addresses, not Solana
                                        if protocol == "PROTOCOL_ETHEREUM" {
                                            if let Some(address) = verification_body.get("address") {
                                                if let Some(addr_str) = address.as_str() {
                                                    addresses.push(addr_str.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            Ok(addresses)
        } else {
            Err(anyhow::anyhow!(
                "Farcaster Hub returned error {}: {}",
                status,
                response_text
            ))
        }
    }

    /// Get the key manager instance
    pub fn key_manager(&self) -> Option<&KeyManager> {
        self.key_manager.as_ref()
    }

    /// Get signers for a FID
    /// 
    /// # Arguments
    /// * `fid` - The Farcaster ID
    /// 
    /// # Returns
    /// * `Result<Vec<SignerInfo>>` - List of signer information or an error
    pub async fn get_signers(&self, fid: u64) -> Result<Vec<SignerInfo>> {
        // Use the Farcaster Hub API to get onchain signers
        let url = format!("{}/v1/onChainSignersByFid?fid={}", self.hub_url, fid);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to get signers from Farcaster Hub")?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            let data: serde_json::Value = serde_json::from_str(&response_text)
                .with_context(|| "Failed to parse signers response")?;
            
            let mut signers = Vec::new();
            
            if let Some(events) = data.get("events").and_then(|e| e.as_array()) {
                for event in events {
                    if let Some(signer_event_body) = event.get("signerEventBody") {
                        if let Some(key) = signer_event_body.get("key").and_then(|k| k.as_str()) {
                            let key_type = signer_event_body.get("keyType")
                                .and_then(|kt| kt.as_u64())
                                .unwrap_or(0);
                            
                            let event_type = signer_event_body.get("eventType")
                                .and_then(|et| et.as_str())
                                .unwrap_or("UNKNOWN");
                            
                            // Only include ADD events (active signers)
                            if event_type == "SIGNER_EVENT_TYPE_ADD" {
                                signers.push(SignerInfo {
                                    key: key.to_string(),
                                    key_type: key_type as u32,
                                    event_type: event_type.to_string(),
                                });
                            }
                        }
                    }
                }
            }
            
            Ok(signers)
        } else {
            anyhow::bail!("❌ Failed to get signers from Farcaster Hub: {} - {}", status, response_text);
        }
    }

    /// Get the hub URL
    pub fn hub_url(&self) -> &str {
        &self.hub_url
    }

    /// Get Ed25519 private key for a FID from local storage
    /// 
    /// # Arguments
    /// * `fid` - The Farcaster ID
    /// 
    /// # Returns
    /// * `Result<Vec<u8>>` - Ed25519 private key bytes or an error
    #[allow(dead_code)]
    async fn get_ed25519_private_key_for_fid(&self, fid: u64) -> Result<Vec<u8>> {
        // Load encrypted Ed25519 key manager
        let keys_file = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::default_keys_file()?;
        let ed25519_manager = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::load_from_file(&keys_file)?;
        
        // Check if Ed25519 key exists for this FID
        if !ed25519_manager.has_key(fid) {
            anyhow::bail!("❌ No Ed25519 key found for FID: {}\n💡 Please generate or import an Ed25519 key for this FID first:\n   castorix hub key generate {}\n   castorix hub key import {}", fid, fid, fid);
        }
        
        // Prompt for password
        let password = crate::encrypted_ed25519_key_manager::prompt_password(&format!("Enter password for FID {fid}: "))?;
        
        // Get the Ed25519 signing key
        let signing_key = ed25519_manager.get_signing_key(fid, &password)?;
        
        Ok(signing_key.to_bytes().to_vec())
    }
}

/// Get Ed25519 public key for a specific FID from encrypted storage
/// 
/// # Arguments
/// * `fid` - The Farcaster ID
/// 
/// # Returns
/// * `Result<String>` - Ed25519 public key in hex format or an error
pub async fn get_ed25519_public_key_for_fid(fid: u64) -> Result<String> {
    // Load encrypted Ed25519 key manager
    let keys_file = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::default_keys_file()?;
    let ed25519_manager = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::load_from_file(&keys_file)?;
    
    // Check if Ed25519 key exists for this FID
    if !ed25519_manager.has_key(fid) {
        anyhow::bail!("❌ No Ed25519 key found for FID: {}\n💡 Please generate or import an Ed25519 key for this FID first:\n   castorix hub key generate {}\n   castorix hub key import {}", fid, fid, fid);
    }
    
    // Get the Ed25519 public key for this FID
    let verifying_key = ed25519_manager.get_verifying_key(fid, "")?;
    Ok(hex::encode(verifying_key.to_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_farcaster_client_creation() {
        let test_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let key_manager = KeyManager::from_private_key(test_key).unwrap();
        let client = FarcasterClient::new("https://hub-api.neynar.com".to_string(), Some(key_manager));
        
        assert_eq!(client.hub_url(), "https://hub-api.neynar.com");
    }

    #[tokio::test]
    async fn test_farcaster_client_from_env() {
        // Set test environment variables
        std::env::set_var("PRIVATE_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
        std::env::set_var("FARCASTER_HUB_URL", "https://hub-api.neynar.com");
        
        let result = FarcasterClient::from_env();
        assert!(result.is_ok());
        
        // Clean up
        std::env::remove_var("PRIVATE_KEY");
        std::env::remove_var("FARCASTER_HUB_URL");
    }
}

