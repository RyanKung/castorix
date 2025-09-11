use ethers::{
    providers::{Provider, Http, Middleware},
    types::{Address, U256, NameOrAddress, transaction::eip2718::TypedTransaction},
};
use anyhow::Result;
use crate::farcaster::contracts::types::{Fid, RecoveryAddress, ContractResult};

/// Id Registry contract wrapper
pub struct IdRegistry {
    provider: Provider<Http>,
    address: Address,
}

impl IdRegistry {
    /// Create a new IdRegistry instance
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
    
    /// Register a new Farcaster ID
    pub async fn register(&self, _recovery: RecoveryAddress) -> Result<ContractResult<Fid>> {
        // This would require a wallet client for actual transaction
        // For now, return an error indicating this is read-only
        Ok(ContractResult::Error("Register requires wallet client - use ID Gateway instead".to_string()))
    }
    
    /// Get the owner of a Farcaster ID
    pub async fn owner_of(&self, fid: Fid) -> Result<ContractResult<Address>> {
        // Use eth_call to read from contract
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
    
    /// Transfer a Farcaster ID to another address
    pub async fn transfer(&self, _to: Address, _fid: Fid) -> Result<ContractResult<()>> {
        Ok(ContractResult::Error("Transfer requires wallet client - use ID Gateway instead".to_string()))
    }
    
    /// Recover a Farcaster ID using the recovery address
    pub async fn recover(&self, _to: Address, _fid: Fid) -> Result<ContractResult<()>> {
        Ok(ContractResult::Error("Recover requires wallet client - use ID Gateway instead".to_string()))
    }
    
    /// Get the FID for a given address
    pub async fn id_of(&self, address: Address) -> Result<ContractResult<Fid>> {
        let data = self.encode_id_of_call(address)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let fid = U256::from_big_endian(&result);
                    Ok(ContractResult::Success(fid.as_u64()))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
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
    
    /// Encode idOf function call
    fn encode_id_of_call(&self, address: Address) -> Result<Vec<u8>> {
        // Function selector for idOf(address): 0x025e7c27
        let mut data = vec![0x02, 0x5e, 0x7c, 0x27];
        let mut address_bytes = [0u8; 32];
        address_bytes[12..32].copy_from_slice(address.as_bytes());
        data.extend_from_slice(&address_bytes);
        Ok(data)
    }
}