use ethers::{
    providers::{Provider, Http, Middleware},
    types::{Address, U256, NameOrAddress, transaction::eip2718::TypedTransaction},
};
use anyhow::Result;
use crate::farcaster::contracts::types::ContractResult;

/// Bundler contract wrapper
pub struct Bundler {
    provider: Provider<Http>,
    address: Address,
}

impl Bundler {
    /// Create a new Bundler instance
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
    
    /// Bundle multiple operations into a single transaction
    pub async fn bundle_operations(&self, _operations: Vec<Vec<u8>>) -> Result<ContractResult<()>> {
        Ok(ContractResult::Error("Bundle operations requires wallet client".to_string()))
    }
    
    /// Get bundler configuration
    pub async fn get_config(&self) -> Result<ContractResult<BundlerConfig>> {
        let data = self.encode_get_config_call()?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 64 {
                    // Parse configuration data
                    let max_operations = u32::from_be_bytes([
                        result[28], result[29], result[30], result[31]
                    ]);
                    let gas_price = U256::from_big_endian(&result[32..64]);
                    
                    Ok(ContractResult::Success(BundlerConfig {
                        max_operations,
                        gas_price,
                    }))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Check if bundler is active
    pub async fn is_active(&self) -> Result<ContractResult<bool>> {
        let data = self.encode_is_active_call()?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let is_active = result[31] != 0;
                    Ok(ContractResult::Success(is_active))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Get bundler fee
    pub async fn get_fee(&self) -> Result<ContractResult<U256>> {
        let data = self.encode_get_fee_call()?;
        let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data.into()),
            ..Default::default()
        });
        match self.provider.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let fee = U256::from_big_endian(&result);
                    Ok(ContractResult::Success(fee))
                } else {
                    Ok(ContractResult::Error("Invalid response length".to_string()))
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Call failed: {e}"))),
        }
    }
    
    /// Encode getConfig function call
    fn encode_get_config_call(&self) -> Result<Vec<u8>> {
        // Function selector for getConfig(): 0x4d2301cc
        let data = vec![0x4d, 0x23, 0x01, 0xcc];
        Ok(data)
    }
    
    /// Encode isActive function call
    fn encode_is_active_call(&self) -> Result<Vec<u8>> {
        // Function selector for isActive(): 0x7f1b4bbd
        let data = vec![0x7f, 0x1b, 0x4b, 0xbd];
        Ok(data)
    }
    
    /// Encode getFee function call
    fn encode_get_fee_call(&self) -> Result<Vec<u8>> {
        // Function selector for getFee(): 0x3ccfd60b
        let data = vec![0x3c, 0xcf, 0xd6, 0x0b];
        Ok(data)
    }
}

/// Bundler configuration
#[derive(Debug, Clone)]
pub struct BundlerConfig {
    pub max_operations: u32,
    pub gas_price: U256,
}
