use crate::cli::types::EnsCommands;
use anyhow::Result;

/// Handle ENS commands
pub async fn handle_ens_command(
    command: EnsCommands,
    ens_proof: &crate::ens_proof::EnsProof,
) -> Result<()> {
    match command {
        EnsCommands::Resolve { domain } => {
            println!("🔍 Resolving ENS domain: {domain}");
            match ens_proof.query_base_ens_contract(&domain).await {
                Ok(Some(address)) => println!("✅ Resolved to: {address}"),
                Ok(None) => println!("❌ Domain not found or not resolved"),
                Err(e) => println!("❌ Failed to resolve domain: {e}"),
            }
        }
        EnsCommands::Domains { address } => {
            println!("🔗 Getting all ENS domains owned by address: {address}");
            match ens_proof.get_all_ens_domains_by_address(&address).await {
                Ok(domains) => {
                    if domains.is_empty() {
                        println!("❌ No ENS domains found for address: {address}");
                    } else {
                        println!("✅ Found {} ENS domain(s):", domains.len());
                        for (i, domain) in domains.iter().enumerate() {
                            let domain_type = if domain.ends_with(".base.eth") {
                                "Base"
                            } else {
                                "ENS"
                            };
                            println!("   {}. {} ({})", i + 1, domain, domain_type);
                        }
                    }
                }
                Err(e) => println!("❌ Failed to get ENS domains: {e}"),
            }
        }
        EnsCommands::BaseSubdomains { address } => {
            println!("🏗️ Getting Base subdomains owned by address: {address}");
            println!("⚠️  Note: Base chain reverse lookup is not currently supported.");
            println!("   Base subdomains are not indexed by The Graph API.");
            println!("   Use 'castorix ens resolve <domain>.base.eth' to check specific domains.");
            match ens_proof.get_base_subdomains_by_address(&address).await {
                Ok(domains) => {
                    if domains.is_empty() {
                        println!("❌ No Base subdomains found for address: {address}");
                    } else {
                        println!("✅ Found {} Base subdomain(s):", domains.len());
                        for (i, domain) in domains.iter().enumerate() {
                            println!("   {}. {}", i + 1, domain);
                        }
                    }
                }
                Err(e) => println!("❌ Failed to get Base subdomains: {e}"),
            }
        }
        EnsCommands::AllDomains { address } => {
            println!("🌐 Getting all ENS domains for address: {address}");
            println!(
                "⚠️  Note: Base subdomains (*.base.eth) reverse lookup is not currently supported."
            );
            match ens_proof.get_all_ens_domains_by_address(&address).await {
                Ok(domains) => {
                    if domains.is_empty() {
                        println!("❌ No domains found for address: {address}");
                    } else {
                        println!("✅ Found {} total domain(s):", domains.len());
                        for (i, domain) in domains.iter().enumerate() {
                            let domain_type = if domain.ends_with(".base.eth") {
                                "Base"
                            } else {
                                "ENS"
                            };
                            println!("   {}. {} ({})", i + 1, domain, domain_type);
                        }
                    }
                }
                Err(e) => println!("❌ Failed to get domains: {e}"),
            }
        }
        EnsCommands::CheckBaseSubdomain { domain } => {
            println!("🔍 Checking Base subdomain: {domain}");
            match ens_proof.check_base_subdomain(&domain).await {
                Ok(Some(owner)) => {
                    println!("✅ Domain exists! Owner: {owner}");
                }
                Ok(None) => {
                    println!("❌ Domain not found in The Graph database");
                }
                Err(e) => println!("❌ Failed to check domain: {e}"),
            }
        }
        EnsCommands::QueryBaseContract { domain } => {
            println!("🔗 Querying Base chain ENS contract for: {domain}");
            match ens_proof.query_base_ens_contract(&domain).await {
                Ok(Some(owner)) => {
                    println!("✅ Domain exists! Owner: {owner}");
                }
                Ok(None) => {
                    println!("❌ Domain not found or contract query failed");
                }
                Err(e) => println!("❌ Failed to query contract: {e}"),
            }
        }
        EnsCommands::Verify { domain } => {
            println!("🔐 Verifying ownership of domain: {domain}");
            match ens_proof.verify_ens_ownership(&domain).await {
                Ok(owned) => {
                    if owned {
                        println!("✅ You own this domain!");
                    } else {
                        println!("❌ You don't own this domain");
                    }
                }
                Err(e) => println!("❌ Failed to verify ownership: {e}"),
            }
        }
        EnsCommands::Create {
            domain,
            fid,
            wallet_name,
        } => {
            if let Some(wallet_name) = &wallet_name {
                println!("📝 Creating username proof for domain: {domain} (FID: {fid}) using wallet: {wallet_name}");
            } else {
                println!("📝 Creating username proof for domain: {domain} (FID: {fid})");
            }
            match ens_proof
                .create_ens_proof_with_wallet(&domain, fid, wallet_name.as_deref())
                .await
            {
                Ok(proof) => {
                    println!("✅ Username proof created successfully!");
                    match ens_proof.serialize_proof(&proof) {
                        Ok(json) => {
                            println!("📄 Proof JSON:");
                            println!("{json}");

                            // Save to file
                            let filename =
                                format!("proof_{}_{}.json", domain.replace(".", "_"), fid);
                            std::fs::write(&filename, &json)?;
                            println!("💾 Proof saved to: {filename}");
                        }
                        Err(e) => println!("❌ Failed to serialize proof: {e}"),
                    }
                }
                Err(e) => println!("❌ Failed to create proof: {e}"),
            }
        }
        EnsCommands::VerifyProof { proof_file } => {
            println!("🔍 Verifying proof from file: {proof_file}");
            let proof_content = std::fs::read_to_string(&proof_file)?;
            let proof_data: serde_json::Value = serde_json::from_str(&proof_content)?;

            // Create UserNameProof from JSON
            let mut proof = crate::core::protocol::username_proof::UserNameProof::new();
            proof.set_timestamp(proof_data["timestamp"].as_u64().unwrap_or(0));
            proof.set_name(
                proof_data["name"]
                    .as_str()
                    .unwrap_or("")
                    .as_bytes()
                    .to_vec(),
            );
            proof.set_owner(hex::decode(proof_data["owner"].as_str().unwrap_or(""))?);
            proof.set_signature(hex::decode(proof_data["signature"].as_str().unwrap_or(""))?);
            proof.set_fid(proof_data["fid"].as_u64().unwrap_or(0));

            match ens_proof.verify_proof(&proof).await {
                Ok(valid) => {
                    if valid {
                        println!("✅ Proof is valid!");
                    } else {
                        println!("❌ Proof is invalid!");
                    }
                }
                Err(e) => println!("❌ Failed to verify proof: {e}"),
            }
        }
    }
    Ok(())
}
