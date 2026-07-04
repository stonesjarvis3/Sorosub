# Deployment Guide

## Prerequisites

### System Requirements
- Rust 1.70+
- Soroban CLI
- Stellar CLI
- Node.js 18+ (optional, for JS SDK)

### Installation

#### Rust & Cargo
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown
```

#### Soroban CLI
```bash
# Install via stellar-cli
cargo install stellar-cli

# Or using package manager (macOS)
brew install stellar-cli
```

#### Verify Installation
```bash
rustc --version
cargo --version
soroban --version
stellar --version
```

## Building the Contract

### 1. Clone & Prepare
```bash
cd sorosub/contracts/sorosub
```

### 2. Build for Soroban
```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled WASM will be at:
```
target/wasm32-unknown-unknown/release/sorosub.wasm
```

### 3. Optimize (Optional but Recommended)
Install wasm-opt if not already present:
```bash
npm install -g wasm-opt
```

Optimize the WASM:
```bash
wasm-opt -Oz target/wasm32-unknown-unknown/release/sorosub.wasm -o target/wasm32-unknown-unknown/release/sorosub-opt.wasm
```

## Testing

### Unit Tests
```bash
cargo test
```

Expected output:
```
test test_subscribe ... ok
test test_subscribe_same_provider_and_subscriber_fails ... ok
test test_duplicate_subscription_fails ... ok
test test_is_payment_due ... ok
test test_time_until_payment ... ok
test test_cancel_subscription ... ok
test test_modify_subscription ... ok
test test_invalid_amount ... ok
test test_invalid_period ... ok

test result: ok. 9 passed
```

### Integration Testing
Use the JS SDK or CLI to test against testnet:

```bash
# Build contract
cargo build --target wasm32-unknown-unknown --release

# Run integration tests (if available)
# Tests would go here
```

## Testnet Deployment

### 1. Setup Network Configuration

Create or use existing Soroban network configuration:

```bash
stellar keys generate admin-key
```

Save the secret key securely. You'll see output like:
```
Public Key: GXXXXXX...
Secret Key: SXXXXXX...
```

### 2. Fund Account

Go to Stellar Testnet Faucet: https://laboratory.stellar.org/

1. Generate keypair and get public key
2. Request testnet XLM funding

### 3. Deploy Contract

```bash
WASM_PATH="target/wasm32-unknown-unknown/release/sorosub.wasm"
CONTRACT_HASH=$(soroban contract install \
  --network testnet \
  --source admin-key \
  --wasm $WASM_PATH)

echo "Contract HASH: $CONTRACT_HASH"
```

Create the contract instance:

```bash
soroban contract deploy \
  --network testnet \
  --source admin-key \
  --salt 0 \
  --wasm-hash $CONTRACT_HASH
```

This returns your CONTRACT_ID:
```
Contract ID: CXXXXXX...
```

### 4. Initialize Contract

```bash
CONTRACT_ID="CXXXXXX..."
ADMIN_KEY_PUB="GXXXXXX..."

soroban contract invoke \
  --network testnet \
  --source admin-key \
  --id $CONTRACT_ID \
  -- initialize \
  --admin $ADMIN_KEY_PUB
```

### 5. Test Functions

Get a test subscriber and provider:

```bash
stellar keys generate subscriber-key
stellar keys generate provider-key

SUBSCRIBER=$(stellar keys address subscriber-key)
PROVIDER=$(stellar keys address provider-key)
USDC_CONTRACT="CBBD...2SAJ"  # Testnet USDC address

# Create subscription
soroban contract invoke \
  --network testnet \
  --source subscriber-key \
  --id $CONTRACT_ID \
  -- subscribe \
  --provider $PROVIDER \
  --subscriber $SUBSCRIBER \
  --token $USDC_CONTRACT \
  --amount 1000000 \
  --period 3600 \
  --name "Test Subscription" \
  --description "Testing SoroSubs on testnet"
```

## Mainnet Deployment

### Pre-Deployment Checklist

- [ ] All tests passing
- [ ] Code reviewed
- [ ] Security audit completed
- [ ] Testnet deployment successful and monitored for 1+ week
- [ ] Documentation finalized
- [ ] Emergency procedures documented
- [ ] Key management plan in place

### 1. Create Mainnet Keys

Use hardware wallet or secure key management:

```bash
# Using Ledger (recommended for mainnet)
stellar keys generate mainnet-admin --hw

# Or local (secure storage)
stellar keys generate mainnet-admin
```

### 2. Fund Account

Send XLM to your mainnet account from an exchange or existing account.

Minimum ~2-5 XLM for contract deployment.

### 3. Deploy to Mainnet

```bash
WASM_PATH="target/wasm32-unknown-unknown/release/sorosub.wasm"

CONTRACT_HASH=$(soroban contract install \
  --network public \
  --source mainnet-admin \
  --wasm $WASM_PATH)

echo "Contract HASH: $CONTRACT_HASH"

soroban contract deploy \
  --network public \
  --source mainnet-admin \
  --salt 0 \
  --wasm-hash $CONTRACT_HASH
```

### 4. Initialize on Mainnet

```bash
CONTRACT_ID="CXXXXXX..."
ADMIN_KEY_PUB="GXXXXXX..."

soroban contract invoke \
  --network public \
  --source mainnet-admin \
  --id $CONTRACT_ID \
  -- initialize \
  --admin $ADMIN_KEY_PUB
```

### 5. Verify Deployment

Check StellarExpert or a block explorer:
```
https://stellar.expert/explorer/public/contract/CXXXXXX...
```

## JavaScript SDK Integration

### Installation

```bash
npm install --save soroban-sdk
```

### Example Usage

```javascript
import { Contract, Client, Networks, StrKey } from "soroban-sdk";

const client = new Client({
  allowHttp: true,
  horizonURL: "https://horizon.stellar.org",
  rpcURL: "https://soroban-testnet.stellar.org:443",
});

const contract = new Contract(CONTRACT_ID, client);

// Subscribe
const tx = await contract.methods
  .subscribe({
    provider: "GXXXXXX...",
    subscriber: "GXXXXXX...",
    token: "CBBD...",
    amount: "1000000",
    period: "2592000",
    name: "Test",
    description: "Testing",
  })
  .simulate();

// Sign and submit
await client.submitTransaction(tx);
```

## Monitoring & Maintenance

### Event Monitoring

Set up indexing for contract events:

```bash
# Query recent events
soroban events \
  --network public \
  --id $CONTRACT_ID \
  --start-ledger 1000000
```

### Payment Processing

Implement a background service to process due payments:

```javascript
// Pseudocode
setInterval(async () => {
  const duePaiements = await queryDueSubscriptions();
  for (const [provider, subscriber] of duePaiements) {
    try {
      await processPayment(provider, subscriber);
    } catch (err) {
      console.error("Payment failed:", err);
      // Alert/log failure
    }
  }
}, 60000); // Every minute
```

### Upgrade Path

For future updates:

1. Deploy new contract version to testnet
2. Test thoroughly
3. Keep old contract running during transition period
4. Migrate users gradually (off-chain coordination)
5. Deprecate old contract after transition

## Troubleshooting

### Contract Won't Deploy
```
Error: Account does not exist
→ Fund your account first
```

```
Error: Transaction failed
→ Check you have enough XLM for fees
```

### Invoke Fails
```
Error: User is not authorized
→ Use correct --source key
```

```
Error: Contract not found
→ Verify CONTRACT_ID
```

### Payment Processing Fails
```
Error: Payment not due yet
→ Check subscription's next_payment timestamp
```

```
Error: Insufficient allowance
→ Set USDC allowance for subscriber
```

## Support

- **Soroban Docs**: https://developers.stellar.org/docs/build/smart-contracts
- **Stellar CLI**: https://github.com/stellar/stellar-cli
- **Stellar Lab**: https://laboratory.stellar.org/
- **Discord**: Stellar Discord community

## Next Steps

1. ✅ Deploy to testnet
2. ✅ Test payment processing
3. ✅ Set up monitoring
4. ✅ Deploy to mainnet
5. ✅ Monitor for issues
6. ✅ Plan future enhancements
