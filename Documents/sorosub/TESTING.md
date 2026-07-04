# Testing Guide

## Unit Tests

### Running Tests

```bash
cd contracts/sorosub
cargo test
```

Due to the authentication requirements in Soroban, unit tests require proper auth setup. The test suite includes basic structure for testing core functionality.

### Test Coverage

Current test structure covers:
- Subscription creation and retrieval
- Time-based payment status checks
- Subscription cancellation
- Subscription modification
- Metadata retrieval
- Non-existent subscription queries

## Integration Testing

### Testnet Testing

Deploy to Soroban testnet and verify functionality end-to-end:

```bash
# 1. Build the contract
cargo build --target wasm32-unknown-unknown --release

# 2. Deploy to testnet (see DEPLOYMENT.md)

# 3. Test subscribe
soroban contract invoke \
  --network testnet \
  --source subscriber-key \
  --id CONTRACT_ID \
  -- subscribe \
  --provider PROVIDER_ADDRESS \
  --subscriber SUBSCRIBER_ADDRESS \
  --token USDC_ADDRESS \
  --amount 1000000 \
  --period 3600 \
  --name "Test Subscription" \
  --description "Testing"

# 4. Query subscription
soroban contract invoke \
  --network testnet \
  --source subscriber-key \
  --id CONTRACT_ID \
  -- get_subscription \
  --provider PROVIDER_ADDRESS \
  --subscriber SUBSCRIBER_ADDRESS

# 5. Check if payment is due
soroban contract invoke \
  --network testnet \
  --id CONTRACT_ID \
  -- is_payment_due \
  --provider PROVIDER_ADDRESS \
  --subscriber SUBSCRIBER_ADDRESS

# 6. Process payment (after waiting for due time)
soroban contract invoke \
  --network testnet \
  --source processor-key \
  --id CONTRACT_ID \
  -- process_payment \
  --provider PROVIDER_ADDRESS \
  --subscriber SUBSCRIBER_ADDRESS
```

### Using the JavaScript SDK

```javascript
import { ContractSpec, Client } from "soroban-sdk";

const spec = new ContractSpec([
  // Contract spec would be generated
]);

const client = new Client(spec, {
  networkPassphrase: "Test SDF Network ; September 2015",
  rpcURL: "https://soroban-testnet.stellar.org:443",
});

// Subscribe
const tx = await client.methods
  .subscribe({
    provider: "GXXXXXX...",
    subscriber: "GXXXXXX...",
    token: "CBBD...",
    amount: "1000000",
    period: "3600",
    name: "Netflix",
    description: "Monthly subscription",
  })
  .simulate();

await client.submitTransaction(tx);

// Check payment due
const isDue = await client.methods
  .is_payment_due({
    provider: "GXXXXXX...",
    subscriber: "GXXXXXX...",
  })
  .call();
```

## Local Testing

### Test Scenarios

#### Scenario 1: Basic Subscription Lifecycle
1. Create subscription with valid parameters
2. Query subscription and verify all fields
3. Check payment is not due yet
4. Modify subscription amount
5. Cancel subscription
6. Verify subscription is inactive

#### Scenario 2: Payment Timing
1. Create subscription with 1-hour period
2. Check payment is not due (should return false)
3. Advance ledger time by 30 minutes
4. Check payment still not due
5. Advance ledger time by 31 more minutes
6. Check payment is now due

#### Scenario 3: Edge Cases
- Same address as provider and subscriber (should fail)
- Duplicate subscriptions (should fail)
- Invalid amounts (zero, too large)
- Invalid periods (too short, too long)
- Operations on inactive subscriptions

#### Scenario 4: Payment Processing
1. Create subscription
2. Verify next_payment is set to now + period
3. Advance time
4. Call process_payment
5. Verify next_payment is updated
6. Verify total_paid is incremented
7. Attempt duplicate processing in same block (should fail)

## Stress Testing

### Large Numbers
```bash
# Test with max amount (1 billion USDC)
amount: 1000000000000000000

# Test with very long period (~10 years)
period: 315360000

# Test with many concurrent subscriptions
# (Create multiple subscriptions with different providers/subscribers)
```

### Performance Testing
- Measure gas cost of each operation
- Monitor storage growth with many subscriptions
- Test concurrent payment processing

## Monitoring & Verification

### Event Verification

After each operation, verify events are emitted:

```bash
soroban events \
  --network testnet \
  --id CONTRACT_ID \
  --topics "sorosub" \
  --start-ledger LEDGER_NUMBER
```

Expected events:
- `subscribe`: When subscription created
- `payment`: When payment processed
- `cancel`: When subscription cancelled
- `modify`: When subscription modified

### State Verification

Query storage to verify internal state:

```bash
soroban contract invoke \
  --network testnet \
  --id CONTRACT_ID \
  -- get_subscription \
  --provider ADDRESS \
  --subscriber ADDRESS
```

Should return subscription with updated fields after each operation.

### Payment Tracking

Implement off-chain service to track payments:

```javascript
const payments = [];

async function trackPayments(provider, subscriber) {
  const events = await queryEvents({
    contractId: CONTRACT_ID,
    topics: ["sorosub", "payment"],
  });

  for (const event of events) {
    if (event.data.provider === provider && 
        event.data.subscriber === subscriber) {
      payments.push({
        amount: event.data.amount,
        timestamp: event.data.timestamp,
      });
    }
  }

  return payments;
}
```

## Debugging

### Common Issues

**Auth Errors**
```
Error(Auth, InvalidAction)
```
Solution: Ensure caller uses `require_auth()` with their keypair

**Subscription Not Found**
```
panicked at 'Subscription not found'
```
Solution: Verify subscription exists before querying

**Invalid Amount**
```
panicked at 'Amount out of valid range'
```
Solution: Amount must be between 1 and 1e18

**Invalid Period**
```
panicked at 'Period out of valid range'
```
Solution: Period must be between 60 seconds and 315,360,000 seconds

### Logging

Enable logging for debugging:

```bash
export SOROBAN_LOG=debug
cargo test -- --nocapture
```

### Event Inspection

Check event logs from failed transactions:

```bash
soroban events \
  --network testnet \
  --start-ledger START \
  --end-ledger END \
  --id CONTRACT_ID
```

## Performance Benchmarks

Expected performance characteristics:

| Operation | Gas Cost | Storage Change |
|-----------|----------|-----------------|
| subscribe | ~5,000 | +300 bytes |
| process_payment | ~3,000 | +8 bytes |
| cancel_subscription | ~1,000 | 0 bytes |
| modify_subscription | ~1,000 | 0 bytes |
| get_subscription | ~500 | 0 bytes |
| is_payment_due | ~500 | 0 bytes |

(Gas estimates based on typical Soroban operations)

## Next Steps

1. ✅ Run unit tests locally
2. ✅ Deploy to testnet
3. ✅ Run integration tests
4. ✅ Monitor events
5. ✅ Stress test with multiple subscriptions
6. ✅ Deploy to mainnet
7. ✅ Monitor mainnet usage
