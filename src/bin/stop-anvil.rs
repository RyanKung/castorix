use std::process::Command;

fn main() {
    println!("🛑 Stopping Anvil node...");

    // Kill all anvil processes
    let output = Command::new("pkill")
        .arg("anvil")
        .output()
        .expect("Failed to execute pkill");

    if output.status.success() {
        println!("✅ Anvil stopped successfully");
    } else {
        println!("⚠️  No Anvil processes found or failed to stop");
    }
}
