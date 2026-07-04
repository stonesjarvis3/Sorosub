# SoroSubs - Recurring USDC Payment Contract

A production-ready Soroban smart contract for automating recurring USDC payments between subscribers and providers (e.g., open-source maintainers).

## Features

- **Recurring Payments**: Set up automated, recurring USDC transfers at fixed intervals
- **Flexible Intervals**: Support for periods from 1 minute to ~10 years
- **Subscriber Control**: Users can modify or cancel subscriptions anytime
- **Payment Tracking**: Total paid amount and payment history tracking
- **Event Logging**: Comprehensive event emission for all contract state changes
- **Input Validation**: Strict validation of amounts, periods, and addresses
- **Abuse Prevention**: Protection against same-block transaction spam
- **Metadata Support**: Store subscription names and descriptions

## Contract Architecture

### Data Structures

#### Subscription
- `provider`: Recipient address (e.g., open-source maintainer)
- `subscriber`: Payer address
- `token`: USDC contract address
- `amount`: Payment amount per period
- `period_seconds`: Interval between payments
- `next_payment`: Timestamp of next due payment
- `is_active`: Whether subscription is currently active
- `created_at`: Subscription creation timestamp
- `total_paid`: Total amount paid to date

#### SubscriptionMetadata
- `name`: Human-readable subscription name
- `description`: Description of the subscription

### Contract Functions

#### `initialize(env, admin)`
Initialize the contract with an admin address. Called once at deployment.

```rust
initialize(env, admin_address)
```

#### `subscribe(...)`
Create a new subscription agreement.

```rust
subscribe(
    env,
    provider,        // Recipient of payments
    subscriber,      // Authenticated payer
    token,          // USDC address
    amount,         // Amount per period
    period,         // Interval in seconds
    name,           // Subscription name
    description     // Subscription description
)
```

**Requirements:**
- Caller must be the subscriber
- No existing subscription between provider and subscriber
- Amount: 1 ≤ amount ≤ 1 billion USDC
- Period: 60 seconds ≤ period ≤ 315,360,000 seconds
- Provider ≠ Subscriber

#### `process_payment(env, provider, subscriber)`
Execute a due payment. Can be called by anyone (e.g., a cron-like service).

```rust
process_payment(env, provider, subscriber)
```

**Requirements:**
- Subscription exists and is active
- Current time ≥ next payment time
- Subscriber has sufficient balance and allowance

#### `cancel_subscription(env, provider, subscriber)`
Deactivate a subscription. Caller must be the subscriber.

```rust
cancel_subscription(env, provider, subscriber)
```

#### `modify_subscription(env, provider, subscriber, new_amount, new_period)`
Modify payment amount and/or interval. Either provider or subscriber can initiate.

```rust
modify_subscription(env, provider, subscriber, new_amount, new_period)
```

**Requirements:**
- Caller must be provider or subscriber (with auth)
- New amount and period within valid ranges

#### `get_subscription(env, provider, subscriber)`
Retrieve subscription details.

```rust
let sub = get_subscription(env, provider, subscriber)
```

Returns `Option<Subscription>`.

#### `get_metadata(env, provider, subscriber)`
Retrieve subscription metadata (name, description).

```rust
let metadata = get_metadata(env, provider, subscriber)
```

#### `is_payment_due(env, provider, subscriber)`
Check if payment is currently due.

```rust
if is_payment_due(env, provider, subscriber) {
    // Payment can be processed
}
```

#### `time_until_payment(env, provider, subscriber)`
Get seconds until next payment. Returns:
- `0` if payment is due now
- `-1` if subscription doesn't exist
- `-1` if subscription is inactive
- Positive number for seconds remaining

```rust
let seconds = time_until_payment(env, provider, subscriber)
```

## Setup & Deployment

### Prerequisites
- Rust 1.70+
- Soroban CLI (`stellar asset soroban install`)
- Node.js 18+ (for JS SDK)

### Build
```bash
cd contracts/sorosub
cargo build --target wasm32-unknown-unknown --release
```

### Test
```bash
cd contracts/sorosub
cargo test
```

### Deploy to Testnet

```bash
# Create a keypair (or use existing)
stellar keys generate my-key

# Deploy the contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/sorosub.wasm \
  --source my-key \
  --network testnet
```

## Events

All state-changing operations emit events:

### SubscriptionCreated
```
{
  provider: Address,
  subscriber: Address,
  amount: i128,
  period_seconds: u64
}
```

### PaymentProcessed
```
{
  provider: Address,
  subscriber: Address,
  amount: i128,
  timestamp: u64
}
```

### SubscriptionCancelled
```
{
  provider: Address,
  subscriber: Address,
  total_paid: i128
}
```

### SubscriptionModified
```
{
  provider: Address,
  subscriber: Address,
  new_amount: i128,
  new_period: u64
}
```

## Security Considerations

### Input Validation
- All amounts are validated to be within MIN_AMOUNT and MAX_AMOUNT
- All periods are validated to be within MIN_PERIOD and MAX_PERIOD
- Provider and subscriber must be different addresses
- Subscription must not already exist before creation

### Abuse Prevention
- Duplicate same-block processing prevented via `LastProcessed` tracking
- No reentrancy vulnerabilities (contract doesn't call user code)
- Token transfers use standard Soroban token interface

### Authorization
- Subscription creation requires subscriber authentication
- Cancellation requires subscriber authentication
- Modification requires either provider or subscriber authentication
- Payment processing can be called by anyone (stateless operation)

## Usage Examples

### Subscribe to a Maintainer's Service
```rust
let provider = Address::from_contract_id(&env, &maintainer_id);
let subscriber = Address::from_contract_id(&env, &my_wallet);
let usdc = Address::from_contract_id(&env, &usdc_contract_id);

subscribe(
    env,
    provider,
    subscriber,
    usdc,
    10_000_000,      // 10 USDC (6 decimals)
    2_592_000,       // 30 days
    String::from_slice(&env, "OpenSSH Donation"),
    String::from_slice(&env, "Monthly support for OpenSSH")
);
```

### Process a Due Payment
```rust
// Check if payment is due
if is_payment_due(&env, provider, subscriber) {
    // Process payment (in production, called by cron service)
    process_payment(&env, provider, subscriber);
}
```

### Cancel a Subscription
```rust
cancel_subscription(&env, provider, subscriber);
```

### Modify Payment Amount
```rust
modify_subscription(&env, provider, subscriber, 20_000_000, 2_592_000);
```

## Limitations & Future Enhancements

### Current Limitations
- Single subscription per provider-subscriber pair
- No built-in fee collection mechanism
- No access control system for admin functions
- Manual payment processing (no oracle/cron integration)

### Potential Enhancements
- Multiple subscriptions per pair (with subscription IDs)
- Tier-based pricing support
- Fee distribution mechanism
- Admin pause/resume functionality
- Integration with Soroban time-lock contracts
- Historical payment querying
- Subscriber-provider messaging

## Testing

The contract includes comprehensive unit tests covering:
- Subscription creation and validation
- Payment processing and state updates
- Cancellation and modification
- Input validation and error cases
- Time-based logic
- Duplicate processing prevention

Run tests with:
```bash
cargo test
```

## License

MIT

## Support

For issues, questions, or contributions, please visit the repository.
