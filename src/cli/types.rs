use clap::Subcommand;

#[derive(Subcommand)]
pub enum KeyCommands {
    /// 📋 Show wallet information
    /// 
    /// Display detailed information about the currently loaded wallet,
    /// including address, public key, and other relevant details.
    Info,
    
    /// ✍️ Sign a message
    /// 
    /// Sign a message with the currently loaded private key.
    /// The signature can be used to prove ownership of the wallet.
    Sign {
        /// Message to sign
        message: String,
    },
    
    /// 🔍 Verify a signature
    /// 
    /// Verify that a signature was created by the wallet owner.
    /// Useful for validating message authenticity.
    Verify {
        /// Message that was signed
        message: String,
        /// Signature to verify (hex string)
        signature: String,
    },
    
    /// 🔐 Generate a new private key (legacy mode)
    /// 
    /// Generate a new private key and display it in plain text.
    /// ⚠️  WARNING: This displays the private key in plain text!
    /// For secure storage, use 'generate-encrypted' instead.
    Generate,
    
    /// 🆕 Generate and encrypt a new private key
    /// 
    /// Create a new private key and store it encrypted with a password.
    /// This is the recommended way to create new wallets securely.
    /// This command will prompt you for the key name and alias.
    /// 
    /// Example: castorix key generate-encrypted
    GenerateEncrypted,
    
    /// 📥 Import and encrypt an existing private key
    /// 
    /// Import an existing private key and store it encrypted with a password.
    /// Perfect for migrating from other wallet software.
    /// This command will prompt you for the private key securely.
    /// 
    /// Example: castorix key import
    Import,
    
    /// 🔓 Load an encrypted private key
    /// 
    /// Load and decrypt a previously stored private key.
    /// You'll be prompted for the password to decrypt the key.
    /// 
    /// Example: castorix key load my-wallet
    Load {
        /// Name of the encrypted key to load
        key_name: String,
    },
    
    /// 📝 List all encrypted keys
    /// 
    /// Display all your encrypted keys with their aliases, addresses,
    /// and creation dates. No passwords required for this operation.
    /// 
    /// Example: castorix key list
    List,
    
    /// 🗑️ Delete an encrypted key
    /// 
    /// Permanently delete an encrypted key from storage.
    /// You'll be prompted for the password to confirm deletion.
    /// ⚠️  WARNING: This action cannot be undone!
    /// 
    /// Example: castorix key delete old-wallet
    Delete {
        /// Name of the encrypted key to delete
        key_name: String,
    },
    
    /// 🔄 Rename an encrypted key
    /// 
    /// Change the filename/identifier of an encrypted key.
    /// The alias and other data remain unchanged.
    /// 
    /// Example: castorix key rename old-name new-name
    Rename {
        /// Current key name
        old_name: String,
        /// New key name
        new_name: String,
    },
    
    /// 🏷️ Update key alias
    /// 
    /// Change the display alias (friendly name) of an encrypted key.
    /// This doesn't affect the filename or other data.
    /// 
    /// Example: castorix key update-alias my-wallet "My Updated Wallet"
    UpdateAlias {
        /// Key name
        key_name: String,
        /// New alias
        new_alias: String,
    },
}

#[derive(Subcommand)]
pub enum HubKeyCommands {
    /// 📋 List all Ed25519 keys
    /// 
    /// Display all your Ed25519 keys with their FIDs and public keys.
    /// 
    /// Example: castorix hub key list
    List,
    
    /// 🆕 Generate a new Ed25519 key pair
    /// 
    /// Create a new Ed25519 key pair for a specific FID and encrypt it with a password.
    /// This key will be used for signing Farcaster messages.
    /// 
    /// Example: castorix hub key generate 12345
    Generate {
        /// FID (Farcaster ID) for this key
        fid: u64,
    },
    
    /// 📥 Import an existing Ed25519 private key
    /// 
    /// Import an existing Ed25519 private key for a specific FID and encrypt it with a password.
    /// Supports both raw Ed25519 keys (32 bytes) and Solana format keys (64 bytes).
    /// Supports hex encoding (with or without 0x prefix) and base58 encoding.
    /// The private key will be prompted interactively to avoid it appearing in shell history.
    /// 
    /// Example: castorix hub key import 12345
    Import {
        /// FID (Farcaster ID) for this key
        fid: u64,
    },
    
    /// 🗑️ Delete an Ed25519 key
    /// 
    /// Permanently delete an Ed25519 key from storage.
    /// ⚠️  WARNING: This action cannot be undone!
    /// 
    /// Example: castorix hub key delete 12345
    Delete {
        /// FID (Farcaster ID) of the key to delete
        fid: u64,
    },
    
    /// 🔍 Show Ed25519 key information
    /// 
    /// Display detailed information about a specific Ed25519 key.
    /// 
    /// Example: castorix hub key info 12345
    Info {
        /// FID (Farcaster ID) of the key
        fid: u64,
    },
    
    /// 🌱 Generate ECDSA key from recovery phrase
    /// 
    /// Generate ECDSA (Custody) key from a recovery phrase (mnemonic) for a specific FID.
    /// This creates the custody key needed for Farcaster account management.
    /// The recovery phrase will be prompted interactively to avoid it appearing in shell history.
    /// 
    /// Note: Ed25519 (Signer) key must be imported separately using 'hub key import'.
    /// 
    /// Example: castorix hub key from-mnemonic 12345
    FromMnemonic {
        /// FID (Farcaster ID) for this key
        fid: u64,
    },
}

#[derive(Subcommand)]
pub enum EnsCommands {
    /// 🔍 Resolve ENS domain to address
    /// 
    /// Look up the Ethereum address associated with an ENS domain.
    /// Useful for verifying domain ownership before creating proofs.
    /// 
    /// Example: castorix ens resolve vitalik.eth
    Resolve {
        /// ENS domain to resolve (e.g., vitalik.eth)
        domain: String,
    },
    
    /// 🔗 Get ENS domains owned by an Ethereum address
    /// 
    /// Query on-chain ENS registry to find all domains owned by a specific address.
    /// This requires an Ethereum RPC URL and may take some time to complete.
    /// 
    /// Example: castorix ens domains 0x1234...
    Domains {
        /// Ethereum address to query
        address: String,
    },
    
    /// 🏗️ Get Base subdomains (*.base.eth) owned by an Ethereum address
    /// 
    /// ⚠️  Note: Base chain reverse lookup is not currently supported.
    /// Base subdomains are not indexed by The Graph API, and direct
    /// contract queries would require enumerating all possible subdomains.
    /// 
    /// Example: castorix ens base-subdomains 0x1234...
    BaseSubdomains {
        /// Ethereum address to query
        address: String,
    },
    
    /// 🌐 Get all ENS domains owned by an Ethereum address
    /// 
    /// Queries for regular ENS domains owned by the address.
    /// Note: Base subdomains (*.base.eth) reverse lookup is not currently supported.
    /// 
    /// Example: castorix ens all-domains 0x1234...
    AllDomains {
        /// Ethereum address to query
        address: String,
    },
    
    /// 🔍 Check if a specific Base subdomain exists and get its owner
    /// 
    /// Check if a specific Base subdomain like ryankung.base.eth exists
    /// and get its owner address.
    /// 
    /// Example: castorix ens check-base-subdomain ryankung.base.eth
    CheckBaseSubdomain {
        /// Base subdomain to check (e.g., ryankung.base.eth)
        domain: String,
    },
    
    /// 🔗 Query Base chain ENS contract directly
    /// 
    /// Query the Base chain ENS contract directly to get domain ownership.
    /// This bypasses The Graph and queries the contract on-chain.
    /// 
    /// Example: castorix ens query-base-contract ryankung.base.eth
    QueryBaseContract {
        /// Base subdomain to query (e.g., ryankung.base.eth)
        domain: String,
    },
    
    /// ✅ Verify ENS domain ownership
    /// 
    /// Check if the currently loaded wallet owns the specified ENS domain.
    /// This is required before creating username proofs.
    /// 
    /// Example: castorix ens verify mydomain.eth
    Verify {
        /// ENS domain to verify
        domain: String,
    },
    
    /// 📝 Create username proof for ENS domain
    /// 
    /// Generate a signed proof linking your ENS domain to your Farcaster ID.
    /// This proof can be submitted to Farcaster to verify domain ownership.
    /// 
    /// Example: castorix ens create mydomain.eth 12345 --wallet-name my-wallet
    Create {
        /// ENS domain name
        domain: String,
        /// Farcaster ID (your FID)
        fid: u64,
        /// Wallet name for encrypted key (optional, uses PRIVATE_KEY if not specified)
        #[arg(long)]
        wallet_name: Option<String>,
    },
    
    /// 🔍 Verify a username proof
    /// 
    /// Verify that a username proof is valid and was signed by the domain owner.
    /// Useful for validating proofs before submission.
    /// 
    /// Example: castorix ens verify-proof proof.json
    VerifyProof {
        /// Path to proof JSON file
        proof_file: String,
    },
}

#[derive(Subcommand)]
pub enum HubCommands {
    /// 👤 Get user information
    /// 
    /// Retrieve detailed information about a Farcaster user by their FID.
    /// Includes profile data, verification status, and other public information.
    /// 
    /// Example: castorix hub user 12345
    User {
        /// Farcaster ID (FID)
        fid: u64,
    },
    
    /// 📝 Submit a cast
    /// 
    /// Post a new cast to Farcaster. Can be a standalone cast or a reply.
    /// Requires a loaded wallet for authentication.
    /// 
    /// Example: castorix hub cast "Hello Farcaster!" 12345
    Cast {
        /// Cast text content
        text: String,
        /// Farcaster ID (your FID)
        fid: u64,
        /// Parent cast ID for replies (format: "fid:hash")
        parent_cast_id: Option<String>,
    },
    
    /// 📤 Submit username proof
    /// 
    /// Submit a previously created username proof to Farcaster Hub.
    /// This links your ENS domain to your Farcaster identity.
    /// 
    /// The system will automatically use the Ed25519 key bound to the specified FID.
    /// If no Ed25519 key exists for the FID, an error will be displayed.
    /// 
    /// Example: castorix hub submit-proof proof.json 12345
    /// Example: castorix hub submit-proof proof.json 12345 --wallet-name my-wallet
    SubmitProof {
        /// Path to proof JSON file
        proof_file: String,
        /// FID (Farcaster ID) for Ed25519 key signing
        fid: u64,
        /// Wallet name for encrypted key (optional, uses PRIVATE_KEY if not specified)
        #[arg(long)]
        wallet_name: Option<String>,
    },

    /// 📤 Submit a username proof to Farcaster Hub using EIP-712 signature
    /// 
    /// Submit a username proof to the Farcaster Hub for verification.
    /// The proof will be signed using EIP-712 signature with the Ethereum private key.
    /// Requires specifying a wallet name for the Ethereum private key.
    /// 
    /// Example: castorix hub submit-proof-eip712 ./proof.json --wallet-name my-wallet
    SubmitProofEip712 {
        /// Path to proof JSON file
        proof_file: String,
        /// Wallet name for encrypted Ethereum private key (required)
        #[arg(long)]
        wallet_name: String,
    },
    
    /// 🔗 Submit Ethereum address verification
    /// 
    /// Submit a verification to link your Ethereum address to your Farcaster account.
    /// This proves ownership of the address and enables ENS integration.
    /// 
    /// Example: castorix hub verify-eth 12345 0x1234...
    VerifyEth {
        /// Farcaster ID (your FID)
        fid: u64,
        /// Ethereum address to verify
        address: String,
    },
    
    /// 🔍 Get Ethereum addresses for a FID
    /// 
    /// Retrieve all Ethereum addresses bound to a specific Farcaster ID.
    /// This is a read-only operation that doesn't require authentication.
    /// 
    /// Example: castorix hub eth-addresses 12345
    EthAddresses {
        /// Farcaster ID (FID)
        fid: u64,
    },
    
    /// 🌐 Get ENS domains with proofs for a FID
    /// 
    /// Retrieve all ENS domains that have proofs for a specific Farcaster ID.
    /// This is a read-only operation that doesn't require authentication.
    /// 
    /// Example: castorix hub ens-domains 12345
    EnsDomains {
        /// Farcaster ID (FID)
        fid: u64,
    },
    
    /// 🏠 Get custody address for a FID
    /// 
    /// Retrieve the custody address (Ethereum address) associated with a specific Farcaster ID.
    /// The custody address is the Ethereum address that registered the FID and has control over it.
    /// This is a read-only operation that doesn't require authentication.
    /// 
    /// Example: castorix hub custody-address 12345
    CustodyAddress {
        /// FID (Farcaster ID) to get custody address for
        fid: u64,
    },

    /// 🔐 Get signers for a FID
    /// 
    /// Retrieve all active signers (account keys) associated with a specific Farcaster ID.
    /// This shows the Ed25519 public keys that are authorized to sign messages for this FID.
    /// This is a read-only operation that doesn't require authentication.
    /// 
    /// Example: castorix hub signers 12345
    Signers {
        /// FID (Farcaster ID) to get signers for
        fid: u64,
    },

    /// 🔑 Ed25519 key management for Farcaster
    /// 
    /// Manage Ed25519 keys used for signing Farcaster messages.
    /// These keys are bound to specific FIDs and used for message authentication.
    Key {
        #[command(subcommand)]
        action: HubKeyCommands,
    },
}
