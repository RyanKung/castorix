use ethers::{
    providers::{Provider, Http, Middleware},
    types::{Address, U256, NameOrAddress, transaction::eip2718::TypedTransaction},
};
use anyhow::Result;
use crate::farcaster::contracts::types::{Fid, StorageUnits, StoragePrice, ContractResult};

/// Storage Registry contract wrapper
pub struct StorageRegistry {
    provider: Provider<Http>,
    address: Address,
}

impl StorageRegistry {
    /// Create a new StorageRegistry instance
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
    
    /// Rent storage units for a Farcaster ID
    pub async fn rent(&self, _fid: Fid, _units: StorageUnits) -> Result<ContractResult<()>> {
        Ok(ContractResult::Error("Rent requires wallet client".to_string()))
    }
    
    /// Get the number of rented units for a Farcaster ID
    pub async fn rented_units(&self, fid: Fid) -> Result<ContractResult<StorageUnits>> {
        let data = self.encode_rented_units_call(fid)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let units = u32::from_be_bytes([
                        result[28], result[29], result[30], result[31]
                    ]);
                    Ok(ContractResult::Success(units))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get the expiry timestamp for rented units
    pub async fn rented_units_expiry(&self, fid: Fid) -> Result<ContractResult<u64>> {
        let data = self.encode_rented_units_expiry_call(fid)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let expiry = U256::from_big_endian(&result);
                    Ok(ContractResult::Success(expiry.as_u64()))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get the price for a specific number of units
    pub async fn price(&self, units: StorageUnits) -> Result<ContractResult<StoragePrice>> {
        let data = self.encode_price_call(units)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let price = U256::from_big_endian(&result);
                    Ok(ContractResult::Success(price))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get the price per unit
    pub async fn price_per_unit(&self) -> Result<ContractResult<StoragePrice>> {
        let data = self.encode_price_per_unit_call()?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let price = U256::from_big_endian(&result);
                    Ok(ContractResult::Success(price))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get the rental period in seconds
    pub async fn rental_period(&self) -> Result<ContractResult<u64>> {
        let data = self.encode_rental_period_call()?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let period = U256::from_big_endian(&result);
                    Ok(ContractResult::Success(period.as_u64()))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Check if a Farcaster ID can rent storage
    pub async fn is_rentable(&self, fid: Fid) -> Result<ContractResult<bool>> {
        let data = self.encode_is_rentable_call(fid)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let rentable = result[31] != 0;
                    Ok(ContractResult::Success(rentable))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Check if a Farcaster ID has active storage rental
    pub async fn is_rented(&self, fid: Fid) -> Result<ContractResult<bool>> {
        let data = self.encode_is_rented_call(fid)?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let rented = result[31] != 0;
                    Ok(ContractResult::Success(rented))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get storage status for a Farcaster ID
    pub async fn get_storage_status(&self, fid: Fid) -> Result<ContractResult<StorageStatus>> {
        let units_result = self.rented_units(fid).await?;
        let expiry_result = self.rented_units_expiry(fid).await?;
        let is_rented_result = self.is_rented(fid).await?;
        
        match (units_result, expiry_result, is_rented_result) {
            (ContractResult::Success(units), ContractResult::Success(expiry), ContractResult::Success(is_rented)) => {
                Ok(ContractResult::Success(StorageStatus {
                    fid,
                    units,
                    expiry,
                    is_rented,
                }))
            }
            (ContractResult::Error(e), _, _) |
            (_, ContractResult::Error(e), _) |
            (_, _, ContractResult::Error(e)) => {
                Ok(ContractResult::Error(format!("Failed to get storage status: {e}")))
            }
        }
    }
    
    /// Calculate the total cost for renting storage units
    pub async fn calculate_rental_cost(&self, units: StorageUnits) -> Result<ContractResult<StoragePrice>> {
        self.price(units).await
    }
    
    /// Encode rentedUnits function call
    fn encode_rented_units_call(&self, fid: Fid) -> Result<Vec<u8>> {
        // Function selector for rentedUnits(uint256): 0x8da5cb5b
        let mut data = vec![0x8d, 0xa5, 0xcb, 0x5b];
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        Ok(data)
    }
    
    /// Encode rentedUnitsExpiry function call
    fn encode_rented_units_expiry_call(&self, fid: Fid) -> Result<Vec<u8>> {
        // Function selector for rentedUnitsExpiry(uint256): 0x8da5cb5b
        let mut data = vec![0x8d, 0xa5, 0xcb, 0x5b];
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        Ok(data)
    }
    
    /// Encode price function call
    fn encode_price_call(&self, units: StorageUnits) -> Result<Vec<u8>> {
        // Function selector for price(uint32): 0x8da5cb5b
        let mut data = vec![0x8d, 0xa5, 0xcb, 0x5b];
        let units_bytes = units.to_be_bytes();
        data.extend_from_slice(&units_bytes);
        Ok(data)
    }
    
    /// Encode pricePerUnit function call
    fn encode_price_per_unit_call(&self) -> Result<Vec<u8>> {
        // Function selector for pricePerUnit(): 0x8da5cb5b
        Ok(vec![0x8d, 0xa5, 0xcb, 0x5b])
    }
    
    /// Encode rentalPeriod function call
    fn encode_rental_period_call(&self) -> Result<Vec<u8>> {
        // Function selector for rentalPeriod(): 0x8da5cb5b
        Ok(vec![0x8d, 0xa5, 0xcb, 0x5b])
    }
    
    /// Encode isRentable function call
    fn encode_is_rentable_call(&self, fid: Fid) -> Result<Vec<u8>> {
        // Function selector for isRentable(uint256): 0x8da5cb5b
        let mut data = vec![0x8d, 0xa5, 0xcb, 0x5b];
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        Ok(data)
    }
    
    /// Encode isRented function call
    fn encode_is_rented_call(&self, fid: Fid) -> Result<Vec<u8>> {
        // Function selector for isRented(uint256): 0x8da5cb5b
        let mut data = vec![0x8d, 0xa5, 0xcb, 0x5b];
        let mut fid_bytes = [0u8; 32];
        U256::from(fid).to_big_endian(&mut fid_bytes);
        data.extend_from_slice(&fid_bytes);
        Ok(data)
    }
}

/// Storage status for a Farcaster ID
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageStatus {
    pub fid: Fid,
    pub units: StorageUnits,
    pub expiry: u64,
    pub is_rented: bool,
}