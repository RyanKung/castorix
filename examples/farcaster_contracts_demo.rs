use castorix::farcaster::contracts::client::FarcasterContractClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Farcaster Contracts Demo");
    
    // 从环境变量创建客户端
    let client = FarcasterContractClient::from_env()?;
    
    println!("✅ 客户端创建成功");
    
    // 获取合约地址
    let addresses = client.addresses();
    println!("📋 合约地址:");
    println!("  ID Gateway: {:?}", addresses.id_gateway);
    println!("  ID Registry: {:?}", addresses.id_registry);
    println!("  Key Gateway: {:?}", addresses.key_gateway);
    println!("  Key Registry: {:?}", addresses.key_registry);
    println!("  Storage Registry: {:?}", addresses.storage_registry);
    
    // 验证合约连接
    println!("\n🔍 验证合约连接...");
    match client.verify_contracts().await {
        Ok(result) => {
            if result.all_working {
                println!("✅ 所有合约连接正常");
            } else {
                println!("⚠️  部分合约连接失败:");
                for error in result.errors {
                    println!("  - {}", error);
                }
                println!("📊 合约状态:");
                println!("  - ID Registry: {}", if result.id_registry { "✅" } else { "❌" });
                println!("  - Key Registry: {}", if result.key_registry { "✅" } else { "❌" });
                println!("  - Storage Registry: {}", if result.storage_registry { "✅" } else { "❌" });
                println!("  - ID Gateway: {}", if result.id_gateway { "✅" } else { "❌" });
                println!("  - Key Gateway: {}", if result.key_gateway { "✅" } else { "❌" });
            }
        }
        Err(e) => {
            println!("❌ 验证失败: {}", e);
        }
    }
    
    // 测试 ID Registry
    println!("\n🔍 测试 ID Registry...");
    match client.id_registry().owner_of(1).await {
        Ok(result) => {
            match result {
                castorix::farcaster::contracts::types::ContractResult::Success(owner) => {
                    println!("✅ FID 1 的拥有者: {:?}", owner);
                }
                castorix::farcaster::contracts::types::ContractResult::Error(e) => {
                    println!("⚠️  查询失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ 调用失败: {}", e);
        }
    }
    
    // 测试 Storage Registry
    println!("\n🔍 测试 Storage Registry...");
    match client.storage_registry().price_per_unit().await {
        Ok(result) => {
            match result {
                castorix::farcaster::contracts::types::ContractResult::Success(price) => {
                    println!("✅ 存储价格: {}", price);
                }
                castorix::farcaster::contracts::types::ContractResult::Error(e) => {
                    println!("⚠️  查询失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ 调用失败: {}", e);
        }
    }
    
    println!("\n🎉 演示完成！");
    Ok(())
}