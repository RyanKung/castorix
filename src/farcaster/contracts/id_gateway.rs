use ethers::{
    providers::{Provider, Http, Middleware},
    types::{Address, U256, NameOrAddress, transaction::eip2718::TypedTransaction},
};
use anyhow::Result;
use crate::farcaster::contracts::types::{Fid, RecoveryAddress, ContractResult};

/// ID Gateway contract wrapper
pub struct IdGateway {
    provider: Provider<Http>,
    address: Address,
}

impl IdGateway {
    /// Create a new IdGateway instance
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
    
    /// Register a new Farcaster ID with extra storage
    pub async fn register(
        &self, 
        _to: Address, 
        _recovery: RecoveryAddress, 
        _extra_storage: U256
    ) -> Result<ContractResult<Fid>> {
        Ok(ContractResult::Error("Register requires wallet client".to_string()))
    }
    
    /// Register a Farcaster ID for another address using signature
    pub async fn register_for(
        &self,
        _to: Address,
        _recovery: RecoveryAddress,
        _extra_storage: U256,
        _deadline: U256,
        _sig: Vec<u8>
    ) -> Result<ContractResult<Fid>> {
        Ok(ContractResult::Error("RegisterFor requires wallet client".to_string()))
    }
    
    /// Get the owner of a Farcaster ID
    pub async fn owner_of(&self, fid: Fid) -> Result<ContractResult<Address>> {
        let data = self.encode_owner_of_call(fid)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let owner = Address::from_slice(&result[12..32]);
                    Ok(ContractResult::Success(owner))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get the recovery address of a Farcaster ID
    pub async fn recovery_of(&self, fid: Fid) -> Result<ContractResult<RecoveryAddress>> {
        let data = self.encode_recovery_of_call(fid)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let recovery = Address::from_slice(&result[12..32]);
                    Ok(ContractResult::Success(recovery))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get the balance of an address (number of FIDs owned)
    pub async fn balance_of(&self, address: Address) -> Result<ContractResult<U256>> {
        let data = self.encode_balance_of_call(address)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let balance = U256::from_big_endian(&result);
                    Ok(ContractResult::Success(balance))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get the total supply of Farcaster IDs
    pub async fn total_supply(&self) -> Result<ContractResult<U256>> {
        let data = self.encode_total_supply_call()?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let supply = U256::from_big_endian(&result);
                    Ok(ContractResult::Success(supply))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get the current ID counter
    pub async fn id_counter(&self) -> Result<ContractResult<Fid>> {
        let data = self.encode_id_counter_call()?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let counter = U256::from_big_endian(&result);
                    Ok(ContractResult::Success(counter.as_u64()))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Check if a Farcaster ID exists
    pub async fn exists(&self, fid: Fid) -> Result<ContractResult<bool>> {
        let data = self.encode_exists_call(fid)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let exists = result[31] != 0;
                    Ok(ContractResult::Success(exists))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get the token URI for a Farcaster ID
    pub async fn token_uri(&self, fid: Fid) -> Result<ContractResult<String>> {
        let data = self.encode_token_uri_call(fid)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 64 {
                    let offset = u32::from_be_bytes([
                        result[28], result[29], result[30], result[31]
                    ]) as usize;
                    
                    if result.len() > offset {
                        let string_data = &result[offset..];
                        if string_data.len() >= 32 {
                            let len = u32::from_be_bytes([
                                string_data[28], string_data[29], string_data[30], string_data[31]
                            ]) as usize;
                            
                            if string_data.len() >= 32 + len {
                                let uri = String::from_utf8_lossy(&string_data[32..32 + len]).to_string();
                                Ok(ContractResult::Success(uri))
                            } else {
                                Ok(ContractResult::Error("Invalid string length".to_string()))
                            }
                        } else {
                            Ok(ContractResult::Error("Invalid string data".to_string()))
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
    
    /// Get comprehensive information about a Farcaster ID
    pub async fn get_fid_info(&self, fid: Fid) -> Result<ContractResult<FidInfo>> {
        let owner_result = self.owner_of(fid).await?;
        let recovery_result = self.recovery_of(fid).await?;
        let exists_result = self.exists(fid).await?;
        let token_uri_result = self.token_uri(fid).await?;
        
        match (owner_result, recovery_result, exists_result, token_uri_result) {
            (
                ContractResult::Success(owner),
                ContractResult::Success(recovery),
                ContractResult::Success(exists),
                ContractResult::Success(token_uri)
            ) => {
                Ok(ContractResult::Success(FidInfo {
                    fid,
                    owner,
                    recovery,
                    exists,
                    token_uri,
                }))
            }
            (ContractResult::Error(e), _, _, _) |
            (_, ContractResult::Error(e), _, _) |
            (_, _, ContractResult::Error(e), _) |
            (_, _, _, ContractResult::Error(e)) => {
                Ok(ContractResult::Error(format!("Failed to get FID info: {e}")))
            }
        }
    }
    
    /// Encode ownerOf function call
    fn encode_owner_of_call(&self, fid: Fid) -> Result<Vec<u8>> {
        // Function selector for ownerOf(uint256): 0x6352211e
        let mut data = vec![0x63, 0x52, 0x21, 0x1e];
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        Ok(data)
    }
    
    /// Encode recoveryOf function call
    fn encode_recovery_of_call(&self, fid: Fid) -> Result<Vec<u8>> {
        // Function selector for recoveryOf(uint256): 0x4f6ccce7
        let mut data = vec![0x4f, 0x6c, 0xcc, 0xe7];
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        Ok(data)
    }
    
    /// Encode balanceOf function call
    fn encode_balance_of_call(&self, address: Address) -> Result<Vec<u8>> {
        // Function selector for balanceOf(address): 0x70a08231
        let mut data = vec![0x70, 0xa0, 0x82, 0x31];
        let mut address_bytes = [0u8; 32];
        address_bytes[12..32].copy_from_slice(address.as_bytes());
        data.extend_from_slice(&address_bytes);
        Ok(data)
    }
    
    /// Encode totalSupply function call
    fn encode_total_supply_call(&self) -> Result<Vec<u8>> {
        // Function selector for totalSupply(): 0x18160ddd
        Ok(vec![0x18, 0x16, 0x0d, 0xdd])
    }
    
    /// Encode idCounter function call
    fn encode_id_counter_call(&self) -> Result<Vec<u8>> {
        // Function selector for idCounter(): 0x8da5cb5b
        Ok(vec![0x8d, 0xa5, 0xcb, 0x5b])
    }
    
    /// Encode exists function call
    fn encode_exists_call(&self, fid: Fid) -> Result<Vec<u8>> {
        // Function selector for exists(uint256): 0x4f6ccce7
        let mut data = vec![0x4f, 0x6c, 0xcc, 0xe7];
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        Ok(data)
    }
    
    /// Encode tokenURI function call
    fn encode_token_uri_call(&self, fid: Fid) -> Result<Vec<u8>> {
        // Function selector for tokenURI(uint256): 0xc87b56dd
        let mut data = vec![0xc8, 0x7b, 0x56, 0xdd];
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        Ok(data)
    }
}

/// Comprehensive FID information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FidInfo {
    pub fid: Fid,
    pub owner: Address,
    pub recovery: RecoveryAddress,
    pub exists: bool,
    pub token_uri: String,
}