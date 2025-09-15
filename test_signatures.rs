use ethers::utils;

fn main() {
    let signatures = vec![
        "Register(uint256,address,address)",
        "Register(address,address,uint256)", 
        "Register(address,address)",
        "IdRegistered(uint256,address,address)",
        "IdRegistered(address,address,uint256)",
        "Transfer(address,address,uint256)",
        "Approval(address,address,uint256)",
    ];
    
    println!("Event signatures:");
    for sig in signatures {
        let hash = utils::keccak256(sig.as_bytes());
        println!("{}: 0x{:x}", sig, hash);
    }
    
    // 从测试输出中看到的实际签名
    println!("\nActual signatures from logs:");
    println!("0xf2e1…7e45: 0xf2e17e45...");
    println!("0xaabd…fe2f: 0xaabd...fe2f");
}
