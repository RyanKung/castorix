pub mod base_ens;
pub mod core;
pub mod query;
pub mod verification;

pub use core::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ens_proof_creation() {
        let test_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let key_manager = crate::key_manager::KeyManager::from_private_key(test_key).unwrap();
        let ens_proof = EnsProof::new(
            key_manager,
            "https://eth-mainnet.g.alchemy.com/v2/test".to_string(),
        );

        // Test proof creation (this will fail domain verification but tests the structure)
        let result = ens_proof.create_ens_proof("test.eth", 123).await;
        // We expect this to fail due to domain verification, but the structure should be correct
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_proof_serialization() {
        let test_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let key_manager = crate::key_manager::KeyManager::from_private_key(test_key).unwrap();
        let ens_proof = EnsProof::new(
            key_manager,
            "https://eth-mainnet.g.alchemy.com/v2/test".to_string(),
        );

        let mut proof = crate::username_proof::UserNameProof::new();
        proof.set_timestamp(1234567890);
        proof.set_name(b"test.eth".to_vec());
        proof.set_owner(b"test_owner".to_vec());
        proof.set_fid(123);
        proof.set_field_type(crate::username_proof::UserNameType::USERNAME_TYPE_ENS_L1);

        let json = ens_proof.serialize_proof(&proof);
        assert!(json.is_ok());
    }

    #[tokio::test]
    async fn test_ens_proof_from_env() {
        // Set test environment variables
        std::env::set_var(
            "PRIVATE_KEY",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
        std::env::set_var("ETH_RPC_URL", "https://eth-mainnet.g.alchemy.com/v2/test");

        let result = EnsProof::from_env();
        assert!(result.is_ok());

        // Clean up
        std::env::remove_var("PRIVATE_KEY");
        std::env::remove_var("ETH_RPC_URL");
    }
}
