#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, String,
};

/// Represents an active subscription agreement
#[contracttype]
#[derive(Clone, Debug)]
pub struct Subscription {
    pub provider: Address,           // Recipient of payments
    pub subscriber: Address,         // Payer of subscription
    pub token: Address,              // USDC contract address
    pub amount: i128,                // Amount per period (in smallest units)
    pub period_seconds: u64,         // Interval between payments (e.g., 2592000 for 30 days)
    pub next_payment: u64,           // Timestamp of next due payment
    pub is_active: bool,             // Whether subscription is currently active
    pub created_at: u64,             // When subscription was created
    pub total_paid: i128,            // Total amount paid so far
}

/// Subscription metadata stored separately for efficiency
#[contracttype]
#[derive(Clone, Debug)]
pub struct SubscriptionMetadata {
    pub name: String,                // Human-readable subscription name
    pub description: String,         // Subscription description
}

/// Storage keys
#[contracttype]
pub enum DataKey {
    Subscription(Address, Address), // (provider, subscriber)
    Metadata(Address, Address),     // (provider, subscriber)
    ActiveSubs(Address),            // List of active subscriptions for provider
    LastProcessed(Address, Address), // (provider, subscriber) - last process timestamp
    AdminAddress,                   // Admin address for emergency controls
}



const MAX_AMOUNT: i128 = 1_000_000_000_000_000_000; // 1 billion USDC (18 decimals)
const MIN_AMOUNT: i128 = 1; // Minimum 0.000000000000000001 USDC
const MAX_PERIOD: u64 = 315_360_000; // ~10 years in seconds
const MIN_PERIOD: u64 = 60; // 1 minute minimum

#[contract]
pub struct SoroSubsContract;

#[contractimpl]
impl SoroSubsContract {
    /// Initialize the contract with an admin address
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();

        let admin_key = DataKey::AdminAddress;
        if env.storage().persistent().has(&admin_key) {
            panic!("Contract already initialized");
        }

        env.storage().persistent().set(&admin_key, &admin);

        soroban_sdk::log!(&env, "SoroSubs contract initialized with admin: {}", admin);
    }

    /// Subscribe to recurring USDC payments
    ///
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `provider` - Address receiving payments (open-source maintainer)
    /// * `subscriber` - Address making payments (authenticated caller)
    /// * `token` - USDC token contract address
    /// * `amount` - Amount to transfer per period (must be > 0 and <= MAX_AMOUNT)
    /// * `period` - Interval between payments in seconds (must be >= MIN_PERIOD and <= MAX_PERIOD)
    /// * `name` - Human-readable name for this subscription
    /// * `description` - Description of what the subscription is for
    pub fn subscribe(
        env: Env,
        provider: Address,
        subscriber: Address,
        token: Address,
        amount: i128,
        period: u64,
        name: String,
        description: String,
    ) {
        // Require subscriber to authenticate this action
        subscriber.require_auth();

        // Validate inputs
        if amount < MIN_AMOUNT || amount > MAX_AMOUNT {
            panic!("Amount out of valid range");
        }
        if period < MIN_PERIOD || period > MAX_PERIOD {
            panic!("Period out of valid range");
        }
        if provider == subscriber {
            panic!("Provider and subscriber must be different");
        }

        let key = DataKey::Subscription(provider.clone(), subscriber.clone());

        // Ensure no existing subscription
        if env.storage().persistent().has(&key) {
            panic!("Subscription already exists");
        }

        let now = env.ledger().timestamp();
        let subscription = Subscription {
            provider: provider.clone(),
            subscriber: subscriber.clone(),
            token,
            amount,
            period_seconds: period,
            next_payment: now + period,
            is_active: true,
            created_at: now,
            total_paid: 0,
        };

        // Store subscription and metadata
        env.storage().persistent().set(&key, &subscription);

        let metadata_key = DataKey::Metadata(provider.clone(), subscriber.clone());
        let metadata = SubscriptionMetadata { name, description };
        env.storage().persistent().set(&metadata_key, &metadata);

        // Emit event
        env.events().publish(
            ("sorosub", "subscribe"),
            (provider.clone(), subscriber.clone(), amount, period),
        );

        soroban_sdk::log!(
            &env,
            "Subscription created: {} -> {} for {} per {}s",
            subscriber,
            provider,
            amount,
            period
        );
    }

    /// Process a due payment from subscriber to provider
    ///
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `provider` - Payment recipient address
    /// * `subscriber` - Payment source address
    ///
    /// # Requirements
    /// * Payment must be due (now >= next_payment_time)
    /// * Subscriber must have approved token allowance
    pub fn process_payment(env: Env, provider: Address, subscriber: Address) {
        let key = DataKey::Subscription(provider.clone(), subscriber.clone());

        let mut subscription: Subscription = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Subscription not found");

        if !subscription.is_active {
            panic!("Subscription is not active");
        }

        let now = env.ledger().timestamp();
        if now < subscription.next_payment {
            panic!("Payment not due yet");
        }

        // Check for spam/abuse - prevent processing same subscription twice in same ledger
        let last_processed_key = DataKey::LastProcessed(provider.clone(), subscriber.clone());
        if let Some(last_timestamp) = env.storage().persistent().get::<_, u64>(&last_processed_key) {
            if last_timestamp == now {
                panic!("Payment already processed in this block");
            }
        }

        // Transfer funds
        let token_client = token::Client::new(&env, &subscription.token);
        token_client.transfer(
            &subscription.subscriber,
            &subscription.provider,
            &subscription.amount,
        );

        // Update subscription state
        subscription.next_payment = now + subscription.period_seconds;
        subscription.total_paid += subscription.amount;
        env.storage().persistent().set(&key, &subscription);

        // Record last processed time
        env.storage()
            .persistent()
            .set(&last_processed_key, &now);

        // Emit event
        env.events().publish(
            ("sorosub", "payment"),
            (provider.clone(), subscriber.clone(), subscription.amount, now),
        );

        soroban_sdk::log!(
            &env,
            "Payment processed: {} -> {} for {}",
            subscriber,
            provider,
            subscription.amount
        );
    }

    /// Cancel an active subscription
    ///
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `provider` - Subscription provider address
    /// * `subscriber` - Subscription subscriber address (authenticated caller)
    pub fn cancel_subscription(env: Env, provider: Address, subscriber: Address) {
        subscriber.require_auth();

        let key = DataKey::Subscription(provider.clone(), subscriber.clone());

        let mut subscription: Subscription = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Subscription not found");

        if !subscription.is_active {
            panic!("Subscription is already inactive");
        }

        subscription.is_active = false;
        env.storage().persistent().set(&key, &subscription);

        // Emit event
        env.events().publish(
            ("sorosub", "cancel"),
            (provider.clone(), subscriber.clone(), subscription.total_paid),
        );

        soroban_sdk::log!(
            &env,
            "Subscription cancelled: {} -> {}",
            subscriber,
            provider
        );
    }

    pub fn modify_subscription(
        env: Env,
        provider: Address,
        subscriber: Address,
        new_amount: i128,
        new_period: u64,
    ) {
        // Validate inputs
        if new_amount < MIN_AMOUNT || new_amount > MAX_AMOUNT {
            panic!("New amount out of valid range");
        }
        if new_period < MIN_PERIOD || new_period > MAX_PERIOD {
            panic!("New period out of valid range");
        }

        let key = DataKey::Subscription(provider.clone(), subscriber.clone());
        let mut subscription: Subscription = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Subscription not found");

        if !subscription.is_active {
            panic!("Cannot modify inactive subscription");
        }

        subscription.amount = new_amount;
        subscription.period_seconds = new_period;
        env.storage().persistent().set(&key, &subscription);

        // Emit event
        env.events().publish(
            ("sorosub", "modify"),
            (provider.clone(), subscriber.clone(), new_amount, new_period),
        );

        soroban_sdk::log!(
            &env,
            "Subscription modified: {} -> {} new amount: {}, new period: {}",
            subscriber,
            provider,
            new_amount,
            new_period
        );
    }

    /// Get subscription details
    pub fn get_subscription(
        env: Env,
        provider: Address,
        subscriber: Address,
    ) -> Option<Subscription> {
        let key = DataKey::Subscription(provider, subscriber);
        env.storage().persistent().get(&key)
    }

    /// Get subscription metadata
    pub fn get_metadata(
        env: Env,
        provider: Address,
        subscriber: Address,
    ) -> Option<SubscriptionMetadata> {
        let key = DataKey::Metadata(provider, subscriber);
        env.storage().persistent().get(&key)
    }

    /// Check if a payment is due
    pub fn is_payment_due(env: Env, provider: Address, subscriber: Address) -> bool {
        let key = DataKey::Subscription(provider, subscriber);
        if let Some(sub) = env.storage().persistent().get::<_, Subscription>(&key) {
            sub.is_active && env.ledger().timestamp() >= sub.next_payment
        } else {
            false
        }
    }

    /// Get time until next payment (in seconds)
    pub fn time_until_payment(env: Env, provider: Address, subscriber: Address) -> i64 {
        let key = DataKey::Subscription(provider, subscriber);
        if let Some(sub) = env.storage().persistent().get::<_, Subscription>(&key) {
            if !sub.is_active {
                return -1; // Subscription inactive
            }
            let now = env.ledger().timestamp();
            if now >= sub.next_payment {
                return 0; // Payment is due
            }
            (sub.next_payment as i64) - (now as i64)
        } else {
            -1 // Subscription doesn't exist
        }
    }
}

#[cfg(test)]
mod test;
