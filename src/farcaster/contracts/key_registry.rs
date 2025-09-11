use ethers::{
    providers::{Provider, Http, Middleware},
    types::{Address, U256, NameOrAddress, transaction::eip2718::TypedTransaction},
};
use anyhow::Result;
use crate::farcaster::contracts::types::{Fid, KeyType, KeyMetadata, ContractResult};

/// Key Registry contract wrapper
pub struct KeyRegistry {
    provider: Provider<Http>,
    address: Address,
}

impl KeyRegistry {
    /// Create a new KeyRegistry instance
    pub fn new(provider: Provider<Http>, address: Address) -> Result<Self> {
        Ok(Self {
            provider,
            address,
        })
    }
    
    /// Get the contract address
    pub fn address(&self) -> Address {
        self.address
    }
    
    /// Add a new key for a Farcaster ID
    pub async fn add(&self, _fid: Fid, _key: Vec<u8>, _key_type: KeyType, _metadata: Vec<u8>) -> Result<ContractResult<()>> {
        Ok(ContractResult::Error("Add requires wallet client - use Key Gateway instead".to_string()))
    }
    
    /// Remove a key for a Farcaster ID
    pub async fn remove(&self, _fid: Fid, _key: Vec<u8>, _key_type: KeyType) -> Result<ContractResult<()>> {
        Ok(ContractResult::Error("Remove requires wallet client - use Key Gateway instead".to_string()))
    }
    
    /// Get key metadata for a specific key
    pub async fn get(&self, fid: Fid, key: Vec<u8>) -> Result<ContractResult<KeyMetadata>> {
        let data = self.encode_get_call(fid, key.clone())?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 64 {
                    let key_type = u32::from_be_bytes([
                        result[28], result[29], result[30], result[31]
                    ]);
                    let metadata_len = u32::from_be_bytes([
                        result[60], result[61], result[62], result[63]
                    ]) as usize;
                    
                    if result.len() >= 64 + metadata_len {
                        let metadata = result[64..64 + metadata_len].to_vec();
                        Ok(ContractResult::Success(KeyMetadata {
                            key_type,
                            key,
                            metadata,
                        }))
                    } else {
                        Ok(ContractResult::Error("Invalid response length".to_string()))
                    }
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get all keys for a Farcaster ID
    pub async fn keys_of(&self, fid: Fid) -> Result<ContractResult<Vec<Vec<u8>>>> {
        let data = self.encode_keys_of_call(fid)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                // Parse dynamic array response
                if result.len() >= 32 {
                    let offset = u32::from_be_bytes([
                        result[28], result[29], result[30], result[31]
                    ]) as usize;
                    
                    if result.len() > offset {
                        let array_data = &result[offset..];
                        if array_data.len() >= 32 {
                            let count = u32::from_be_bytes([
                                array_data[28], array_data[29], array_data[30], array_data[31]
                            ]) as usize;
                            
                            let mut keys = Vec::new();
                            let mut pos = 32;
                            
                            for _ in 0..count {
                                if pos + 32 <= array_data.len() {
                                    let key_offset = u32::from_be_bytes([
                                        array_data[pos + 28], array_data[pos + 29], 
                                        array_data[pos + 30], array_data[pos + 31]
                                    ]) as usize;
                                    
                                    if key_offset < array_data.len() {
                                        let key_len = u32::from_be_bytes([
                                            array_data[key_offset + 28], array_data[key_offset + 29],
                                            array_data[key_offset + 30], array_data[key_offset + 31]
                                        ]) as usize;
                                        
                                        if key_offset + 32 + key_len <= array_data.len() {
                                            let key = array_data[key_offset + 32..key_offset + 32 + key_len].to_vec();
                                            keys.push(key);
                                        }
                                    }
                                    pos += 32;
                                }
                            }
                            
                            Ok(ContractResult::Success(keys))
                        } else {
                            Ok(ContractResult::Error("Invalid array data".to_string()))
                        }
                    } else {
                        Ok(ContractResult::Error("Invalid offset".to_string()))
                    }
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get the number of keys for a Farcaster ID
    pub async fn key_count_of(&self, fid: Fid) -> Result<ContractResult<u32>> {
        let data = self.encode_key_count_of_call(fid)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let count = u32::from_be_bytes([
                        result[28], result[29], result[30], result[31]
                    ]);
                    Ok(ContractResult::Success(count))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get all key metadata for a Farcaster ID
    pub async fn get_all_keys(&self, fid: Fid) -> Result<ContractResult<Vec<KeyMetadata>>> {
        let keys_result = self.keys_of(fid).await?;
        match keys_result {
            ContractResult::Success(keys) => {
                let mut key_metadata = Vec::new();
                for key in keys {
                    match self.get(fid, key.clone()).await? {
                        ContractResult::Success(metadata) => {
                            key_metadata.push(KeyMetadata {
                                key_type: metadata.key_type,
                                key,
                                metadata: metadata.metadata,
                            });
                        }
                        ContractResult::Error(e) => {
                            return Ok(ContractResult::Error(format!("Failed to get key metadata: {e}")));
                        }
                    }
                }
                Ok(ContractResult::Success(key_metadata))
            }
            ContractResult::Error(e) => Ok(ContractResult::Error(e)),
        }
    }
    
    /// Encode get function call
    fn encode_get_call(&self, fid: Fid, key: Vec<u8>) -> Result<Vec<u8>> {
        // Function selector for get(uint256,bytes): 0x8da5cb5b
        let mut data = vec![0x8d, 0xa5, 0xcb, 0x5b];
        
        // Add fid parameter
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        
        // Add key parameter (dynamic bytes)
        let key_offset = 32 + 32; // fid + offset
        let key_offset_bytes = (key_offset as u32).to_be_bytes();
        data.extend_from_slice(&key_offset_bytes);
        
        // Add key length and data
        let key_len_bytes = (key.len() as u32).to_be_bytes();
        data.extend_from_slice(&key_len_bytes);
        data.extend_from_slice(&key);
        
        Ok(data)
    }
    
    /// Encode keysOf function call
    fn encode_keys_of_call(&self, fid: Fid) -> Result<Vec<u8>> {
        // Function selector for keysOf(uint256): 0x8da5cb5b
        let mut data = vec![0x8d, 0xa5, 0xcb, 0x5b];
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        Ok(data)
    }
    
    /// Encode keyCountOf function call
    fn encode_key_count_of_call(&self, fid: Fid) -> Result<Vec<u8>> {
        // Function selector for keyCountOf(uint256): 0x8da5cb5b
        let mut data = vec![0x8d, 0xa5, 0xcb, 0x5b];
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        Ok(data)
    }
}