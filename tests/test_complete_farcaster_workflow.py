#!/usr/bin/env python3
"""
Complete Farcaster Workflow Integration Test

This test covers the full Farcaster workflow:
1. Generate and register a key
2. Use that key as payment for FID registration
3. Test storage rental
4. Test signer registration and deletion
5. Test FID listing and storage usage queries

The test uses Python's pexpect library to handle interactive CLI commands.
"""

import os
import sys
import time
import subprocess
import tempfile
import shutil
import json
import pexpect
from pathlib import Path
from typing import Optional, Tuple


class FarcasterWorkflowTest:
    def __init__(self):
        self.test_dir = Path("./test_data")
        self.keys_dir = self.test_dir / "keys"
        self.wallet_name = "test-workflow-wallet"
        self.password = "testpassword123"
        self.anvil_process: Optional[subprocess.Popen] = None
        
    def setup(self):
        """Set up test environment"""
        print("🚀 Starting Complete Farcaster Workflow Test")
        
        # Clean up previous test data
        if self.test_dir.exists():
            shutil.rmtree(self.test_dir)
        
        # Create test directories
        self.test_dir.mkdir(exist_ok=True)
        self.keys_dir.mkdir(exist_ok=True)
        
        # Start Anvil node
        self.start_anvil()
        
        # Create a wallet first (this will be used throughout the test)
        self.create_initial_wallet()
    
    def create_initial_wallet(self):
        """Create the initial wallet that will be used throughout the test"""
        print("   🔑 Creating initial test wallet...")
        
        # Generate a random private key for testing
        import secrets
        private_key = "0x" + secrets.token_hex(32)
        print(f"   📝 Generated test private key: {private_key[:10]}...")
        
        # Set environment variable for the CLI
        env = os.environ.copy()
        env["PRIVATE_KEY"] = private_key
        
        # Run wallet creation command with interactive inputs
        inputs = [
            self.wallet_name,  # Key name
            "y",               # Confirm encryption
            self.password,     # Password
            self.password      # Confirm password
        ]
        
        exit_code, stdout, stderr = self.run_cli_command(
            ["key", "generate-encrypted"], 
            inputs=inputs
        )
        
        if exit_code == 0 and "✅" in stdout and ("created" in stdout or "saved" in stdout):
            print("   ✅ Initial wallet created successfully")
            # Extract wallet address
            for line in stdout.split('\n'):
                if "Address:" in line:
                    print(f"   📝 {line.strip()}")
                    break
        else:
            print(f"   ❌ Initial wallet creation failed")
            print(f"   📝 stdout: {stdout}")
            print(f"   📝 stderr: {stderr}")
            raise Exception("Failed to create initial wallet")
        
    def teardown(self):
        """Clean up test environment"""
        if self.anvil_process:
            self.anvil_process.terminate()
            self.anvil_process.wait()
        
        # Clean up test data
        if self.test_dir.exists():
            shutil.rmtree(self.test_dir)
    
    def start_anvil(self):
        """Start local Anvil node for testing"""
        print("📡 Starting local Anvil node...")
        
        try:
            # Start Anvil process
            self.anvil_process = subprocess.Popen([
                "anvil",
                "--host", "127.0.0.1",
                "--port", "8545",
                "--chain-id", "31337",
                "--gas-limit", "30000000",
                "--gas-price", "1000000000"
            ], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            
            # Wait for Anvil to start
            time.sleep(3)
            
            # Test RPC connection
            result = subprocess.run([
                "curl", "-s", "-X", "POST",
                "-H", "Content-Type: application/json",
                "-d", '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}',
                "http://127.0.0.1:8545"
            ], capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0:
                print("✅ Anvil RPC is responding")
            else:
                raise Exception("Anvil RPC not responding")
                
            print("✅ Anvil is running")
            
        except Exception as e:
            print(f"❌ Failed to start Anvil: {e}")
            raise
    
    def run_cli_command(self, args: list, inputs: list = None, timeout: int = 60) -> Tuple[int, str, str]:
        """
        Run a CLI command with optional interactive inputs
        
        Args:
            args: Command arguments (excluding 'cargo run --bin castorix')
            inputs: List of inputs to send interactively
            timeout: Command timeout in seconds
            
        Returns:
            Tuple of (exit_code, stdout, stderr)
        """
        cmd = ["cargo", "run", "--bin", "castorix", "--", "--path", str(self.test_dir)] + args
        
        if inputs is None:
            # Non-interactive command
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
            return result.returncode, result.stdout, result.stderr
        else:
            # Interactive command using pexpect
            try:
                child = pexpect.spawn(" ".join(cmd), timeout=timeout, env=os.environ.copy())
                child.logfile_read = sys.stdout.buffer
                
                output = ""
                for i, input_text in enumerate(inputs):
                    # Wait for prompt and send input
                    child.expect([pexpect.EOF, pexpect.TIMEOUT, "Enter", "Do you want", "password"])
                    if child.before:
                        output += child.before.decode('utf-8', errors='ignore')
                    
                    if i < len(inputs):
                        child.sendline(input_text)
                
                # Wait for completion
                child.expect(pexpect.EOF, timeout=timeout)
                if child.before:
                    output += child.before.decode('utf-8', errors='ignore')
                
                return child.exitstatus or 0, output, ""
                
            except pexpect.TIMEOUT:
                child.terminate()
                return 1, output, "Command timed out"
            except Exception as e:
                return 1, "", str(e)
    
    def test_fid_price_query(self):
        """Test FID price query"""
        print("   💰 Testing FID price query...")
        
        exit_code, stdout, stderr = self.run_cli_command(["fid", "price"])
        
        if exit_code == 0 and "Base Registration Price" in stdout:
            print("   ✅ FID price query successful")
            # Extract price information
            for line in stdout.split('\n'):
                if "Base Registration Price" in line:
                    print(f"   📊 {line.strip()}")
                    break
            return True
        else:
            print(f"   ❌ FID price query failed: {stderr}")
            return False
    
    def test_wallet_creation(self):
        """Test wallet creation with interactive input"""
        print("   🔑 Testing wallet creation...")
        
        # Wallet was already created in setup, just verify it exists
        exit_code, stdout, stderr = self.run_cli_command(["key", "list"])
        
        if exit_code == 0 and self.wallet_name in stdout:
            print("   ✅ Wallet creation verified successfully")
            return True
        else:
            print(f"   ❌ Wallet creation verification failed")
            print(f"   📝 stdout: {stdout}")
            print(f"   📝 stderr: {stderr}")
            return False
    
    def test_fid_registration(self):
        """Test FID registration using the created wallet"""
        print("   🆕 Testing FID registration...")
        
        # Run FID registration with the created wallet (interactive password input)
        exit_code, stdout, stderr = self.run_cli_command([
            "fid", "register", 
            "--wallet", self.wallet_name,
            "--yes"
        ], inputs=[self.password])
        
        if exit_code == 0 and "✅" in stdout:
            print("   ✅ FID registration successful")
            # Extract registration result
            for line in stdout.split('\n'):
                if "FID" in line and ("registered" in line or "created" in line):
                    print(f"   📝 {line.strip()}")
                    break
            return True
        else:
            print(f"   ❌ FID registration failed")
            print(f"   📝 stdout: {stdout}")
            print(f"   📝 stderr: {stderr}")
            return False
    
    def test_storage_rental(self):
        """Test storage rental"""
        print("   💾 Testing storage rental...")
        
        # First get FID list to find the registered FID
        exit_code, stdout, stderr = self.run_cli_command(["fid", "list"])
        
        if exit_code != 0:
            print("   ❌ Failed to get FID list for storage test")
            return False
        
        # Extract FID from the list (assuming we have one)
        fid = None
        for line in stdout.split('\n'):
            if "FID:" in line:
                try:
                    fid = int(line.split("FID:")[1].strip().split()[0])
                    break
                except (ValueError, IndexError):
                    continue
        
        if fid is None:
            print("   ⚠️  No FID found for storage test")
            return False
        
        print(f"   📝 Using FID {fid} for storage test")
        
        # Test storage rental
        exit_code, stdout, stderr = self.run_cli_command([
            "storage", "rent",
            "--fid", str(fid),
            "--wallet", self.wallet_name,
            "--units", "1",
            "--yes"
        ])
        
        if exit_code == 0 and "✅" in stdout:
            print("   ✅ Storage rental successful")
            return True
        else:
            print(f"   ❌ Storage rental failed")
            print(f"   📝 stdout: {stdout}")
            print(f"   📝 stderr: {stderr}")
            return False
    
    def test_signer_registration(self):
        """Test signer registration and deletion"""
        print("   ✍️  Testing signer registration...")
        
        # Get FID for signer test
        exit_code, stdout, stderr = self.run_cli_command(["fid", "list"])
        
        if exit_code != 0:
            print("   ❌ Failed to get FID list for signer test")
            return False
        
        fid = None
        for line in stdout.split('\n'):
            if "FID:" in line:
                try:
                    fid = int(line.split("FID:")[1].strip().split()[0])
                    break
                except (ValueError, IndexError):
                    continue
        
        if fid is None:
            print("   ⚠️  No FID found for signer test")
            return False
        
        # Test signer registration
        exit_code, stdout, stderr = self.run_cli_command([
            "signers", "register",
            "--fid", str(fid),
            "--wallet", self.wallet_name,
            "--yes"
        ])
        
        if exit_code == 0 and "✅" in stdout:
            print("   ✅ Signer registration successful")
        else:
            print(f"   ❌ Signer registration failed")
            print(f"   📝 stdout: {stdout}")
            print(f"   📝 stderr: {stderr}")
            return False
        
        # Test signer deletion
        print("   🗑️  Testing signer deletion...")
        exit_code, stdout, stderr = self.run_cli_command([
            "signers", "delete",
            "--fid", str(fid),
            "--wallet", self.wallet_name,
            "--yes"
        ])
        
        if exit_code == 0 and "✅" in stdout:
            print("   ✅ Signer deletion successful")
            return True
        else:
            print(f"   ❌ Signer deletion failed")
            print(f"   📝 stdout: {stdout}")
            print(f"   📝 stderr: {stderr}")
            return False
    
    def test_fid_listing_and_storage_usage(self):
        """Test FID listing and storage usage queries"""
        print("   📋 Testing FID listing and storage usage...")
        
        # Test FID list
        exit_code, stdout, stderr = self.run_cli_command(["fid", "list"])
        
        if exit_code == 0:
            print("   ✅ FID listing successful")
            # Show FID information
            for line in stdout.split('\n'):
                if "FID:" in line or "Address:" in line:
                    print(f"   📝 {line.strip()}")
        else:
            print(f"   ❌ FID listing failed: {stderr}")
            return False
        
        # Test storage usage
        exit_code, stdout, stderr = self.run_cli_command(["storage", "usage"])
        
        if exit_code == 0:
            print("   ✅ Storage usage query successful")
            # Show storage information
            for line in stdout.split('\n'):
                if "Storage" in line or "Units" in line:
                    print(f"   📝 {line.strip()}")
        else:
            print(f"   ❌ Storage usage query failed: {stderr}")
            return False
        
        return True
    
    def run_complete_test(self):
        """Run the complete Farcaster workflow test"""
        try:
            self.setup()
            
            print("\n🆕 Testing FID Registration...")
            
            # Test 1: FID price query
            if not self.test_fid_price_query():
                return False
            
            # Test 2: Wallet creation (the critical interactive part)
            if not self.test_wallet_creation():
                return False
            
            # Test 3: FID registration using the created wallet
            if not self.test_fid_registration():
                return False
            
            print("\n💾 Testing Storage Operations...")
            
            # Test 4: Storage rental
            if not self.test_storage_rental():
                return False
            
            print("\n✍️  Testing Signer Operations...")
            
            # Test 5: Signer registration and deletion
            if not self.test_signer_registration():
                return False
            
            print("\n📋 Testing Query Operations...")
            
            # Test 6: FID listing and storage usage
            if not self.test_fid_listing_and_storage_usage():
                return False
            
            print("\n🎉 Complete Farcaster Workflow Test PASSED!")
            print("✅ All interactive wallet creation and registration tests completed successfully")
            
            return True
            
        except Exception as e:
            print(f"\n❌ Test failed with exception: {e}")
            return False
        finally:
            self.teardown()


def main():
    """Main test function"""
    # Check if pexpect is available
    try:
        import pexpect
    except ImportError:
        print("❌ pexpect library not found. Please install it with:")
        print("   pip install pexpect")
        sys.exit(1)
    
    # Check if anvil is available
    try:
        subprocess.run(["anvil", "--help"], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("❌ anvil not found. Please install foundry:")
        print("   curl -L https://foundry.paradigm.xyz | bash")
        print("   foundryup")
        sys.exit(1)
    
    # Run the test
    test = FarcasterWorkflowTest()
    success = test.run_complete_test()
    
    if success:
        print("\n🎉 All tests passed!")
        sys.exit(0)
    else:
        print("\n❌ Some tests failed!")
        sys.exit(1)


if __name__ == "__main__":
    main()
