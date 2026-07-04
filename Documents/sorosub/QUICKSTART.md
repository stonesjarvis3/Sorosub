# SoroSubs Quick Start

Get up and running with SoroSubs in 5 minutes.

## Installation & Build

```bash
# Clone or navigate to the project
cd sorosub

# Build for Soroban
cd contracts/sorosub
cargo build --target wasm32-unknown-unknown --release

# Output WASM binary
ls -lh target/wasm32-unknown-unknown/release/sorosub.wasm
```

## Deploy to Testnet

### 1. Generate Keys
```bash
stellar keys generate my-admin-key
stellar keys generate provider-key
stellar keys generate subscriber-key
```

### 2. Fund Accounts
Visit Stellar Testnet Faucet: https://laboratory.stellar.org/
- Get your public key: `stellar keys address my-admin-key`
- Request testnet XLM funding

### 3. Deploy Contract
```bash
WASM_PATH="target/wasm32-unknown-unknown/release/sorosub.wasm"

CONTRACT_HASH=$(soroban contract install \
  --network testnet \
  --source my-admin-key \
  --wasm $WASM_PATH)

CONTRACT_ID=$(soroban contract deploy \
  --network testnet \
  --source my-admin-key \
  --salt 0 \
  --wasm-hash $CONTRACT_HASH)

echo "Contract ID: $CONTRACT_ID"
```

### 4. Initialize
```bash
ADMIN_KEY=$(stellar keys address my-admin-key)

soroban contract invoke \
  --network testnet \
  --source my-admin-key \
  --id $CONTRACT_ID \
  -- initialize \
  --admin $ADMIN_KEY
```

## Create Your First Subscription

### 1. Get Addresses
```bash
PROVIDER=$(stellar keys address provider-key)
SUBSCRIBER=$(stellar keys address subscriber-key)
USDC_CONTRACT="CBBD77AB4CIRJTRBN4LZVICCWXTN7IERIASMKCPE"  # Testnet USDC
```

### 2. Subscribe
```bash
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
  --name "My Subscription" \
  --description "Testing SoroSubs"
```

### 3. Query Subscription
```bash
soroban contract invoke \
  --network testnet \
  --id $CONTRACT_ID \
  -- get_subscription \
  --provider $PROVIDER \
  --subscriber $SUBSCRIBER
```

### 4. Check Payment Status
```bash
soroban contract invoke \
  --network testnet \
  --id $CONTRACT_ID \
  -- is_payment_due \
  --provider $PROVIDER \
  --subscriber $SUBSCRIBER

# Should return: false (payment not due yet)
```

## Core Commands Reference

### Subscribe
```bash
soroban contract invoke \
  --network testnet \
  --source subscriber-key \
  --id $CONTRACT_ID \
  -- subscribe \
  --provider PROVIDER_ADDRESS \
  --subscriber SUBSCRIBER_ADDRESS \
  --token USDC_ADDRESS \
  --amount 1000000 \
  --period 3600 \
  --name "Service Name" \
  --description "Description"
```

### Process Payment
```bash
soroban contract invoke \
  --network testnet \
  --id $CONTRACT_ID \
  -- process_payment \
  --provider PROVIDER_ADDRESS \
  --subscriber SUBSCRIBER_ADDRESS
```

### Cancel Subscription
```bash
soroban contract invoke \
  --network testnet \
  --source subscriber-key \
  --id $CONTRACT_ID \
  -- cancel_subscription \
  --provider PROVIDER_ADDRESS \
  --subscriber SUBSCRIBER_ADDRESS
```

### Modify Subscription
```bash
soroban contract invoke \
  --network testnet \
  --source subscriber-key \
  --id $CONTRACT_ID \
  -- modify_subscription \
  --provider PROVIDER_ADDRESS \
  --subscriber SUBSCRIBER_ADDRESS \
  --new-amount 2000000 \
  --new-period 7200
```

### Get Metadata
```bash
soroban contract invoke \
  --network testnet \
  --id $CONTRACT_ID \
  -- get_metadata \
  --provider PROVIDER_ADDRESS \
  --subscriber SUBSCRIBER_ADDRESS
```

### Check Time Until Payment
```bash
soroban contract invoke \
  --network testnet \
  --id $CONTRACT_ID \
  -- time_until_payment \
  --provider PROVIDER_ADDRESS \
  --subscriber SUBSCRIBER_ADDRESS
```

## JavaScript SDK Usage

```javascript
import { SorobanClient } from "soroban-sdk";

const client = new SorobanClient({
  networkPassphrase: "Test SDF Network ; September 2015",
  rpcURL: "https://soroban-testnet.stellar.org:443",
});

// Subscribe
await client.contract(CONTRACT_ID)
  .call("subscribe", {
    provider: "GXXXXXX...",
    subscriber: "GXXXXXX...",
    token: "CBBD...",
    amount: 1000000n,
    period: 3600n,
    name: "Netflix",
    description: "Monthly subscription",
  })
  .sign()
  .submit();

// Check if payment is due
const isDue = await client.contract(CONTRACT_ID)
  .call("is_payment_due", {
    provider: "GXXXXXX...",
    subscriber: "GXXXXXX...",
  })
  .call();

console.log("Payment due:", isDue);
```

## Testing Locally

```bash
cd contracts/sorosub

# Run tests
cargo test

# With logs
RUST_LOG=debug cargo test -- --nocapture
```

## Common Parameters

- **amount**: Satoshis (1 USDC = 1,000,000). Example: `1000000` = 1 USDC
- **period**: Seconds. Examples:
  - `3600` = 1 hour
  - `86400` = 1 day
  - `2592000` = 30 days
  - `31536000` = 1 year
- **provider**: Recipient address (open-source maintainer)
- **subscriber**: Payer address (your account)

## Troubleshooting

### "InvalidAction" Error
```
Error(Auth, InvalidAction)
```
**Solution**: Make sure you're using `--source` with the correct key

### "Subscription already exists"
```
panicked at 'Subscription already exists'
```
**Solution**: A subscription already exists for this provider-subscriber pair

### "Payment not due yet"
```
panicked at 'Payment not due yet'
```
**Solution**: Payment is scheduled for later. Check `time_until_payment`

### "Subscription not found"
```
panicked at 'Subscription not found'
```
**Solution**: Create a subscription first

## Next Steps

1. ✅ Deploy on testnet
2. ✅ Create test subscriptions
3. ✅ Process payments
4. ✅ Read documentation (README.md, ARCHITECTURE.md)
5. ✅ Plan mainnet deployment (see DEPLOYMENT.md)
6. ✅ Deploy on mainnet when ready

## Resources

- **README.md**: Full feature documentation
- **DEPLOYMENT.md**: Detailed deployment guide
- **ARCHITECTURE.md**: System design & storage patterns
- **SECURITY.md**: Security considerations
- **TESTING.md**: Testing procedures
- **PRODUCTION_READY.md**: Pre-launch checklist

## Support

For issues or questions:
1. Check the troubleshooting guide above
2. Review relevant documentation
3. Check contract events: `soroban events --network testnet --id CONTRACT_ID`
4. Enable debug logging: `RUST_LOG=debug`
