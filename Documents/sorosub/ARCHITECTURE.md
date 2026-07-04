# SoroSubs Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Soroban Smart Contract                       │
│                      (SoroSubsContract)                          │
└─────────────────────────────────────────────────────────────────┘
         ↓                           ↓                      ↓
    ┌─────────┐          ┌──────────────────┐      ┌────────────┐
    │ Storage │          │  Token Interface │      │   Events   │
    │ (Persistent)       │   (USDC Transfer)       │ (Indexed   │
    └─────────┘          └──────────────────┘      │  Off-chain)│
         ↓                           ↓              └────────────┘
    Subscriptions         Token Contract
    Metadata              (Stellar USDC)
    AdminAddress
```

## State Management

### Storage Structure

```
DataKey::Subscription(provider: Address, subscriber: Address)
  → Subscription {
    provider: Address,
    subscriber: Address,
    token: Address,
    amount: i128,
    period_seconds: u64,
    next_payment: u64,
    is_active: bool,
    created_at: u64,
    total_paid: i128,
  }

DataKey::Metadata(provider: Address, subscriber: Address)
  → SubscriptionMetadata {
    name: String,
    description: String,
  }

DataKey::LastProcessed(provider: Address, subscriber: Address)
  → u64 (timestamp of last payment processing)

DataKey::AdminAddress
  → Address
```

### Access Patterns

**Write Operations:**
- `subscribe()`: Creates new Subscription + Metadata
- `process_payment()`: Updates Subscription (next_payment, total_paid), LastProcessed
- `cancel_subscription()`: Sets is_active = false
- `modify_subscription()`: Updates amount and period_seconds

**Read Operations:**
- `get_subscription()`: Retrieves Subscription
- `get_metadata()`: Retrieves Metadata
- `is_payment_due()`: Reads Subscription to check timing
- `time_until_payment()`: Calculates based on Subscription

## Function Call Flows

### Subscribe Flow

```
subscriber.require_auth()
    ↓
validate inputs (amount, period, addresses)
    ↓
check subscription doesn't exist
    ↓
create Subscription object with now + period as next_payment
    ↓
store Subscription and Metadata
    ↓
emit SubscriptionCreated event
```

### Process Payment Flow

```
retrieve Subscription
    ↓
verify is_active
    ↓
check now >= next_payment
    ↓
check not already processed in this block
    ↓
call token.transfer(subscriber → provider, amount)
    ↓
update next_payment and total_paid
    ↓
store updated Subscription
    ↓
record LastProcessed timestamp
    ↓
emit PaymentProcessed event
```

### Cancel Subscription Flow

```
subscriber.require_auth()
    ↓
retrieve Subscription
    ↓
verify is_active
    ↓
set is_active = false
    ↓
store updated Subscription
    ↓
emit SubscriptionCancelled event
```

### Modify Subscription Flow

```
verify caller is provider or subscriber
    ↓
if subscriber: subscriber.require_auth()
if provider: provider.require_auth()
    ↓
validate new amount and period
    ↓
retrieve Subscription
    ↓
verify is_active
    ↓
update amount and period_seconds
    ↓
store updated Subscription
    ↓
emit SubscriptionModified event
```

## Security Design

### Authentication Model

- **Subscriber Authentication Required**: Subscribe, Cancel
- **Either-Or Authentication**: Modify (provider OR subscriber)
- **No Authentication Required**: Process Payment (stateless operation)
- **Initial Setup**: Initialize (admin only, one-time)

### Authorization Model

```
┌─ Subscriber ─┐
│              ├─→ Create subscription
│              ├─→ Cancel subscription
│              ├─→ Modify subscription (requires auth)
└──────────────┘

┌─ Provider ───┐
│              ├─→ Modify subscription (requires auth)
└──────────────┘

┌─ Anyone ─────┐
│              ├─→ Process payment (no auth required)
│              ├─→ Query subscription
│              ├─→ Check payment due
└──────────────┘
```

### Abuse Prevention

**Double-Processing Protection**
- Track `LastProcessed` timestamp per subscription
- Prevent processing twice in same ledger block
- Implements check: `if last_processed == now { panic!() }`

**Input Validation**
- Amount bounds: 1 ≤ amount ≤ 1e18
- Period bounds: 60 ≤ period ≤ 315,360,000
- Unique key constraint: (provider, subscriber) must be unique
- Self-subscription prevented: provider ≠ subscriber

**Reentrancy Safety**
- No callbacks or hooks to user code
- Token transfer is only external call
- State modified after transfer completes
- Standard token interface (no custom behavior)

## Scalability Considerations

### Storage Layout

**Per-Subscription Storage: ~2 persistent entries**
- 1 × Subscription struct (~200 bytes)
- 1 × Metadata struct (~100 bytes)
- 1 × LastProcessed (8 bytes, lazily created)

**Lookup: O(1)** via composite key (provider, subscriber)

**Growth Model:**
- n subscriptions = ~300 bytes × n
- 1 million subscriptions ≈ 300 GB (not a concern for Soroban)

### Transaction Costs

**Per Operation:**
- `subscribe()`: 1 write (Subscription) + 1 write (Metadata) + 1 event
- `process_payment()`: 1 read, 1 write, 1 token transfer, 1 event
- `cancel_subscription()`: 1 read, 1 write, 1 event
- `modify_subscription()`: 1 read, 1 write, 1 event

**Optimization:** Read-only queries (get_subscription, is_payment_due) have minimal cost

### Payment Processing Model

**Stateless Design**: `process_payment` can be called by anyone, anytime
- No batching needed
- Horizontally scalable
- Off-chain service calls it reliably
- Multiple concurrent calls safe (checked by `LastProcessed`)

## Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                    User Interactions                              │
└──────────────────────────────────────────────────────────────────┘
     ↓                    ↓                    ↓              ↓
┌─────────────┐    ┌────────────┐    ┌──────────────┐  ┌──────────┐
│  Subscribe  │    │   Process  │    │  Modify/    │  │  Query   │
│             │    │  Payment   │    │ Cancel      │  │          │
└─────────────┘    └────────────┘    └──────────────┘  └──────────┘
     ↓                    ↓                    ↓              ↓
  Auth Check          Read Sub            Auth Check      Read Only
  Validate            Transfer            Validate
  Unique Key          Update State        Update State
     ↓                    ↓                    ↓              ↓
┌───────────────────────────────────────────────────────────────────┐
│                    Soroban Storage Layer                           │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │ Persistent Storage (RocksDB):                            │   │
│  │  • Subscriptions                                         │   │
│  │  • Metadata                                              │   │
│  │  • Last Processed Timestamps                             │   │
│  │  • Admin Address                                         │   │
│  └────────────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────────────┘
     ↓
┌───────────────────────────────────────────────────────────────────┐
│                        Event Stream                               │
│  • SubscriptionCreated                                            │
│  • PaymentProcessed                                               │
│  • SubscriptionCancelled                                          │
│  • SubscriptionModified                                           │
└───────────────────────────────────────────────────────────────────┘
     ↓
┌───────────────────────────────────────────────────────────────────┐
│            Off-Chain Indexing & Monitoring                        │
│  • Event indexers (Stellar Indexer, Quicknode, etc.)             │
│  • Payment processor service                                      │
│  • User dashboards                                                │
│  • Analytics                                                      │
└───────────────────────────────────────────────────────────────────┘
```

## Extension Points

### Future Enhancements

**1. Fee Collection**
```rust
struct FeeConfig {
    protocol_fee_percent: u32,  // 0.1% = 10
    fee_recipient: Address,
}

// In process_payment:
let fee = amount * protocol_fee_percent / 10000;
let net = amount - fee;
token.transfer(subscriber, provider, net);
token.transfer(subscriber, fee_recipient, fee);
```

**2. Tier-Based Subscriptions**
```rust
pub struct Tier {
    id: u32,
    name: String,
    amount: i128,
    benefits: String,
}

// Store multiple tiers per provider
pub enum DataKey {
    Tier(Address, u32),  // (provider, tier_id)
    ...
}
```

**3. Payment History**
```rust
pub struct Payment {
    timestamp: u64,
    amount: i128,
    status: PaymentStatus,
}

pub enum DataKey {
    PaymentHistory(Address, Address, u64),  // (provider, subscriber, index)
    ...
}
```

**4. Admin Controls**
```rust
pub fn pause_subscription(env: Env, provider: Address, subscriber: Address) {
    // Allow admin to pause (not for users)
}

pub fn resume_subscription(env: Env, provider: Address, subscriber: Address) {
    // Allow admin to resume
}
```

## Performance Characteristics

| Operation | Time | Storage | Notes |
|-----------|------|---------|-------|
| subscribe | O(1) | +300B | Validates, creates 2 entries |
| process_payment | O(1) | +8B | Token transfer is main cost |
| cancel_subscription | O(1) | 0 | Just flips flag |
| modify_subscription | O(1) | 0 | Overwrites existing |
| get_subscription | O(1) | 0 | Read-only |
| is_payment_due | O(1) | 0 | Single check |
| time_until_payment | O(1) | 0 | Calculation |

All operations are O(1) due to composite key lookup.
