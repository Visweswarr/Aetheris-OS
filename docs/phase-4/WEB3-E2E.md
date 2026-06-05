# Web3 End-to-End Testing Guide

## Overview

This document provides comprehensive instructions for testing the complete Web3 contract deployment and interaction flow in Aetheris OS Phase 4. The testing covers wallet creation, contract deployment, method calls, and state queries using the `netctl` CLI tool.

## Prerequisites

### Required Tools
- **netctl**: Aetheris OS network control CLI tool
- **Go 1.22+**: For building netctl
- **Contract files**: HelloWorld.sol contract

### Build netctl
```bash
# Build the netctl CLI tool
cd go/tooling/netctl
go build -o netctl .

# Verify netctl is available
./netctl --help
```

### Contract File
Ensure the HelloWorld.sol contract exists at `contracts/HelloWorld.sol`:

```solidity
// contracts/HelloWorld.sol
pragma solidity ^0.8.0;

contract HelloWorld {
    string public greeting;
    
    constructor() {
        greeting = "Hello, World!";
    }
    
    function setGreeting(string memory _greeting) public {
        greeting = _greeting;
    }
    
    function getGreeting() public view returns (string memory) {
        return greeting;
    }
}
```

## Manual Testing Steps

### Step 1: Create Wallet

Create a new wallet using the secp256k1 algorithm:

```bash
# Create wallet with secp256k1 algorithm
netctl wallet create --algo secp256k1 --out seeds/wallet.json

# Verify wallet creation
ls -la seeds/wallet.json
cat seeds/wallet.json
```

**Expected Output:**
```json
{
  "address": "0x...",
  "private_key": "...",
  "public_key": "...",
  "algorithm": "secp256k1"
}
```

**Verification:**
- ✅ Wallet file created at `seeds/wallet.json`
- ✅ Contains address, private_key, public_key fields
- ✅ Algorithm field set to "secp256k1"

### Step 2: Deploy Contract

Deploy the HelloWorld contract using the created wallet:

```bash
# Deploy HelloWorld contract
netctl contract deploy contracts/HelloWorld.sol --from seeds/wallet.json --label hello

# Verify deployment
netctl contract list
```

**Expected Output:**
```
Contract deployed successfully
Label: hello
Address: 0x...
Transaction Hash: 0x...
Gas Used: 123456
```

**Verification:**
- ✅ Contract deployed successfully
- ✅ Label "hello" assigned
- ✅ Contract address returned
- ✅ Transaction hash provided
- ✅ Gas usage reported

### Step 3: Call Contract Method

Call the `setGreeting` method to update the greeting value:

```bash
# Call setGreeting method with "Aetheris"
netctl contract call hello setGreeting "Aetheris" --from seeds/wallet.json

# Verify method call
netctl contract call hello setGreeting "Aetheris" --from seeds/wallet.json --dry-run
```

**Expected Output:**
```
Method call successful
Contract: hello
Method: setGreeting
Arguments: ["Aetheris"]
Transaction Hash: 0x...
Gas Used: 23456
```

**Verification:**
- ✅ Method call executed successfully
- ✅ Arguments passed correctly
- ✅ Transaction hash returned
- ✅ Gas usage reported

### Step 4: Query Contract State

Query the current greeting value from the contract:

```bash
# Query greeting value
netctl contract query hello greeting | tee artifacts/contract_greeting.txt

# Alternative query method
netctl contract query hello getGreeting | tee artifacts/contract_greeting.txt
```

**Expected Output:**
```
Query successful
Contract: hello
Method: greeting
Result: "Aetheris"
```

**Verification:**
- ✅ Query executed successfully
- ✅ Result shows "Aetheris" (the value we set)
- ✅ Output saved to `artifacts/contract_greeting.txt`

### Step 5: Validate Results

Verify the complete contract interaction flow:

```bash
# Check the query output file
cat artifacts/contract_greeting.txt

# Verify the greeting value
grep -o '"Aetheris"' artifacts/contract_greeting.txt
```

**Expected Result:**
```
"Aetheris"
```

**Final Validation:**
- ✅ Greeting value is "Aetheris" as expected
- ✅ Contract state persistence confirmed
- ✅ End-to-end flow completed successfully

## Automated Testing

### Run Smoke Test Script

Use the provided smoke test script for automated testing:

```bash
# Run the complete smoke test
bash scripts/p4-contracts-smoke.sh
```

**Expected Output:**
```
🚀 P4 Contracts Smoke Test
==========================

📋 Checking prerequisites...
✅ Prerequisites check passed
📋 Setting up test environment...
✅ Test environment ready
📋 Step 1: Creating wallet with secp256k1 algorithm...
✅ Wallet created successfully
📋 Step 2: Deploying HelloWorld contract...
✅ Contract deployed successfully with label 'hello'
📋 Step 3: Calling setGreeting method with 'Aetheris'...
✅ setGreeting method called successfully
📋 Step 4: Querying greeting value...
✅ Greeting query executed successfully
📋 Step 5: Validating contract interaction results...
✅ Contract interaction validation passed
PASS: Contract greeting value is 'Aetheris' as expected

🎉 P4 Contracts Smoke Test PASSED

Summary:
  ✅ Wallet created with secp256k1 algorithm
  ✅ HelloWorld contract deployed successfully
  ✅ setGreeting method called with 'Aetheris'
  ✅ Greeting value queried and verified
  ✅ Contract state persistence confirmed

Artifacts generated:
  📄 seeds/wallet.json (wallet file)
  📄 artifacts/contract_greeting.txt (query output)
```

## Troubleshooting

### Common Issues

#### 1. netctl Command Not Found
**Error:** `netctl: command not found`

**Solution:**
```bash
# Build netctl
cd go/tooling/netctl
go build -o netctl .

# Add to PATH or use full path
export PATH=$PATH:$(pwd)
# OR
./netctl --help
```

#### 2. Contract File Not Found
**Error:** `HelloWorld.sol contract not found`

**Solution:**
```bash
# Verify contract file exists
ls -la contracts/HelloWorld.sol

# Create contract file if missing
mkdir -p contracts
# Copy the HelloWorld.sol content from above
```

#### 3. Wallet Creation Failed
**Error:** `Failed to create wallet`

**Solution:**
```bash
# Check directory permissions
ls -la seeds/

# Create seeds directory
mkdir -p seeds

# Check disk space
df -h
```

#### 4. Contract Deployment Failed
**Error:** `Failed to deploy contract`

**Solution:**
```bash
# Verify wallet file
cat seeds/wallet.json

# Check contract syntax
# Ensure HelloWorld.sol is valid Solidity

# Check network connectivity
netctl network status
```

#### 5. Method Call Failed
**Error:** `Failed to call setGreeting method`

**Solution:**
```bash
# Verify contract is deployed
netctl contract list

# Check method signature
netctl contract methods hello

# Verify wallet has sufficient balance
netctl wallet balance seeds/wallet.json
```

#### 6. Query Failed
**Error:** `Failed to query greeting value`

**Solution:**
```bash
# Verify contract state
netctl contract state hello

# Check method availability
netctl contract methods hello

# Verify contract is active
netctl contract status hello
```

### Debug Commands

#### Check Contract Status
```bash
# List all contracts
netctl contract list

# Get contract details
netctl contract info hello

# Check contract methods
netctl contract methods hello

# View contract state
netctl contract state hello
```

#### Check Wallet Status
```bash
# List wallets
netctl wallet list

# Check wallet balance
netctl wallet balance seeds/wallet.json

# Verify wallet integrity
netctl wallet verify seeds/wallet.json
```

#### Check Network Status
```bash
# Network status
netctl network status

# Connection info
netctl network info

# Test connectivity
netctl network ping
```

## Advanced Testing

### Multiple Contract Interactions

Test multiple contract deployments and interactions:

```bash
# Deploy multiple contracts
netctl contract deploy contracts/HelloWorld.sol --from seeds/wallet.json --label hello1
netctl contract deploy contracts/HelloWorld.sol --from seeds/wallet.json --label hello2

# Interact with multiple contracts
netctl contract call hello1 setGreeting "Hello from Contract 1" --from seeds/wallet.json
netctl contract call hello2 setGreeting "Hello from Contract 2" --from seeds/wallet.json

# Query both contracts
netctl contract query hello1 greeting
netctl contract query hello2 greeting
```

### Error Handling Testing

Test error conditions:

```bash
# Test invalid method call
netctl contract call hello invalidMethod "test" --from seeds/wallet.json

# Test invalid contract label
netctl contract call invalidLabel setGreeting "test" --from seeds/wallet.json

# Test invalid wallet
netctl contract call hello setGreeting "test" --from invalid/wallet.json
```

### Performance Testing

Test contract interaction performance:

```bash
# Time contract deployment
time netctl contract deploy contracts/HelloWorld.sol --from seeds/wallet.json --label perf-test

# Time method calls
time netctl contract call perf-test setGreeting "Performance Test" --from seeds/wallet.json

# Time queries
time netctl contract query perf-test greeting
```

## Integration with CI/CD

### GitHub Actions Integration

Add to your CI workflow:

```yaml
- name: Run Web3 Contract Tests
  run: |
    bash scripts/p4-contracts-smoke.sh
    
- name: Upload Contract Test Artifacts
  uses: actions/upload-artifact@v3
  with:
    name: contract-test-artifacts
    path: |
      seeds/wallet.json
      artifacts/contract_greeting.txt
```

### Local Development

For local development testing:

```bash
# Run tests in development mode
export AETHERIS_ENV=development
bash scripts/p4-contracts-smoke.sh

# Run tests with verbose output
bash scripts/p4-contracts-smoke.sh --verbose

# Run tests with custom seed
export AETHERIS_SEED=12345
bash scripts/p4-contracts-smoke.sh
```

## Success Criteria

The Web3 end-to-end test is considered successful when:

1. ✅ **Wallet Creation**: Wallet created with secp256k1 algorithm
2. ✅ **Contract Deployment**: HelloWorld contract deployed successfully
3. ✅ **Method Call**: setGreeting method called with "Aetheris"
4. ✅ **State Query**: Greeting value queried and verified
5. ✅ **State Persistence**: Contract state persists between calls
6. ✅ **Result Validation**: Final greeting value is "Aetheris"

## Next Steps

After successful Web3 testing:

1. **Integration Testing**: Test contract integration with other Aetheris OS services
2. **Performance Testing**: Benchmark contract deployment and interaction performance
3. **Security Testing**: Test contract security and access control
4. **Multi-User Testing**: Test contract interactions with multiple users
5. **Cross-Chain Testing**: Test contract deployment across different blockchain networks

This comprehensive testing ensures the Web3 contract functionality is working correctly and provides a foundation for more advanced blockchain integration in Aetheris OS.
