use anyhow::Result;
use castorix::consts::get_config;
use castorix::farcaster::contracts::FarcasterContractClient;
use ethers::providers::Middleware;
use ethers::types::{transaction::eip2718::TypedTransaction, Address, NameOrAddress, U256};

/// Test different contract interfaces and function selectors
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Farcaster Contract Interface Test");
    println!("===================================\n");

    let config = get_config();
    let client =
        FarcasterContractClient::new_with_default_addresses(config.eth_op_rpc_url().to_string())?;

    let provider = client.provider();
    let addresses = client.addresses();

    println!("📋 Contract Addresses:");
    println!("  ID Registry: {:?}", addresses.id_registry);
    println!("  Key Registry: {:?}", addresses.key_registry);
    println!();

    // Test different function selectors for ID Registry
    println!("🔍 Testing ID Registry function selectors:");

    // Test 1: Standard ERC721 ownerOf (0x6352211e)
    println!("\n1. Testing standard ERC721 ownerOf (0x6352211e):");
    test_function_call(
        provider,
        addresses.id_registry,
        &[0x63, 0x52, 0x21, 0x1e],
        1u64,
    )
    .await?;

    // Test 2: Alternative ownerOf selector
    println!("\n2. Testing alternative ownerOf selector:");
    test_function_call(
        provider,
        addresses.id_registry,
        &[0x8f, 0x4e, 0xb6, 0x5c],
        1u64,
    )
    .await?;

    // Test 3: Simple balanceOf call (0x70a08231)
    println!("\n3. Testing balanceOf (0x70a08231):");
    test_function_call(
        provider,
        addresses.id_registry,
        &[0x70, 0xa0, 0x82, 0x31],
        1u64,
    )
    .await?;

    // Test 4: Try a different approach - check if it's a standard contract
    println!("\n4. Testing if contract responds to basic calls:");
    test_basic_contract_call(provider, addresses.id_registry).await?;

    // Test Key Registry with different selectors
    println!("\n🔑 Testing Key Registry function selectors:");

    // Test 1: keyCountOf
    println!("\n1. Testing keyCountOf:");
    test_function_call(
        provider,
        addresses.key_registry,
        &[0x4f, 0x6c, 0xcc, 0xe7],
        1u64,
    )
    .await?;

    // Test 2: Alternative key count function
    println!("\n2. Testing alternative key count function:");
    test_function_call(
        provider,
        addresses.key_registry,
        &[0x8f, 0x4e, 0xb6, 0x5c],
        1u64,
    )
    .await?;

    println!("\n🎉 Contract interface test completed!");
    println!("💡 Check the response lengths to understand the contract interface.");

    Ok(())
}

async fn test_function_call(
    provider: &ethers::providers::Provider<ethers::providers::Http>,
    address: Address,
    selector: &[u8],
    param: u64,
) -> Result<()> {
    let mut data = selector.to_vec();
    let mut param_bytes = [0u8; 32];
    U256::from(param).to_big_endian(&mut param_bytes);
    data.extend_from_slice(&param_bytes);

    let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
        to: Some(NameOrAddress::Address(address)),
        data: Some(data.into()),
        ..Default::default()
    });

    match provider.call(&tx, None).await {
        Ok(result) => {
            println!("  ✅ Response length: {} bytes", result.len());
            if result.len() >= 32 {
                println!("  📊 First 32 bytes: {}", hex::encode(&result[..32]));
                if result.len() > 32 {
                    println!("  📊 Additional bytes: {}", hex::encode(&result[32..]));
                }
            } else {
                println!("  📊 Full response: {}", hex::encode(&result));
            }
        }
        Err(e) => {
            println!("  ❌ Call failed: {}", e);
        }
    }

    Ok(())
}

async fn test_basic_contract_call(
    provider: &ethers::providers::Provider<ethers::providers::Http>,
    address: Address,
) -> Result<()> {
    // Test with no data (just see if contract responds)
    let tx = TypedTransaction::Legacy(ethers::types::TransactionRequest {
        to: Some(NameOrAddress::Address(address)),
        data: Some(vec![].into()),
        ..Default::default()
    });

    match provider.call(&tx, None).await {
        Ok(result) => {
            println!("  ✅ Contract responds to empty call");
            println!("  📊 Response: {}", hex::encode(&result));
        }
        Err(e) => {
            println!("  ❌ Contract doesn't respond to empty call: {}", e);
        }
    }

    Ok(())
}
