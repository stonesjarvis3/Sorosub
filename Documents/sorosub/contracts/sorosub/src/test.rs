#[cfg(test)]
mod tests {
    use crate::{SoroSubsContract, SoroSubsContractClient};
    use soroban_sdk::{
        testutils::{Address as AddressTestUtils, Ledger},
        Address, Env, String,
    };

    #[test]
    fn test_subscribe_and_query() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSubsContract);
        let client = SoroSubsContractClient::new(&env, &contract_id);

        let provider = Address::generate(&env);
        let subscriber = Address::generate(&env);
        let token = Address::generate(&env);
        let amount = 1_000_000i128;
        let period = 2_592_000u64;

        let name = String::from_str(&env, "Netflix");
        let description = String::from_str(&env, "Monthly subscription");

        // Call subscribe without auth - simulates on-chain behavior
        client.subscribe(
            &provider,
            &subscriber,
            &token,
            &amount,
            &period,
            &name,
            &description,
        );

        // Query the subscription
        let sub = client.get_subscription(&provider, &subscriber);
        assert!(sub.is_some());

        let sub = sub.unwrap();
        assert_eq!(sub.provider, provider);
        assert_eq!(sub.subscriber, subscriber);
        assert_eq!(sub.amount, amount);
        assert_eq!(sub.period_seconds, period);
        assert!(sub.is_active);
        assert_eq!(sub.total_paid, 0);
    }

    #[test]
    fn test_is_payment_due() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSubsContract);
        let client = SoroSubsContractClient::new(&env, &contract_id);

        let provider = Address::generate(&env);
        let subscriber = Address::generate(&env);
        let token = Address::generate(&env);
        let period = 2_592_000u64;

        client.subscribe(
            &provider,
            &subscriber,
            &token,
            &1_000_000,
            &period,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "Test"),
        );

        // Payment should not be due immediately
        assert!(!client.is_payment_due(&provider, &subscriber));

        // Advance time past next payment
        env.ledger().set_timestamp(period + 1);

        // Now payment should be due
        assert!(client.is_payment_due(&provider, &subscriber));
    }

    #[test]
    fn test_time_until_payment() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSubsContract);
        let client = SoroSubsContractClient::new(&env, &contract_id);

        let provider = Address::generate(&env);
        let subscriber = Address::generate(&env);
        let token = Address::generate(&env);
        let period = 2_592_000u64;

        env.ledger().set_timestamp(0);

        client.subscribe(
            &provider,
            &subscriber,
            &token,
            &1_000_000,
            &period,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "Test"),
        );

        let time_until = client.time_until_payment(&provider, &subscriber);
        assert_eq!(time_until, period as i64);

        // Advance time halfway
        env.ledger().set_timestamp(period / 2);
        let time_until = client.time_until_payment(&provider, &subscriber);
        assert_eq!(time_until, (period / 2) as i64);
    }

    #[test]
    fn test_cancel_subscription() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSubsContract);
        let client = SoroSubsContractClient::new(&env, &contract_id);

        let provider = Address::generate(&env);
        let subscriber = Address::generate(&env);
        let token = Address::generate(&env);

        client.subscribe(
            &provider,
            &subscriber,
            &token,
            &1_000_000,
            &2_592_000,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "Test"),
        );

        assert!(client.get_subscription(&provider, &subscriber).unwrap().is_active);

        client.cancel_subscription(&provider, &subscriber);

        assert!(!client.get_subscription(&provider, &subscriber).unwrap().is_active);
    }

    #[test]
    fn test_modify_subscription() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSubsContract);
        let client = SoroSubsContractClient::new(&env, &contract_id);

        let provider = Address::generate(&env);
        let subscriber = Address::generate(&env);
        let token = Address::generate(&env);
        let new_amount = 2_000_000i128;
        let new_period = 1_296_000u64;

        client.subscribe(
            &provider,
            &subscriber,
            &token,
            &1_000_000,
            &2_592_000,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "Test"),
        );

        client.modify_subscription(&provider, &subscriber, &new_amount, &new_period);

        let sub = client.get_subscription(&provider, &subscriber).unwrap();
        assert_eq!(sub.amount, new_amount);
        assert_eq!(sub.period_seconds, new_period);
    }

    #[test]
    fn test_get_metadata() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSubsContract);
        let client = SoroSubsContractClient::new(&env, &contract_id);

        let provider = Address::generate(&env);
        let subscriber = Address::generate(&env);
        let token = Address::generate(&env);
        let name = String::from_str(&env, "MyService");
        let description = String::from_str(&env, "A great service");

        client.subscribe(
            &provider,
            &subscriber,
            &token,
            &1_000_000,
            &2_592_000,
            &name,
            &description,
        );

        let metadata = client.get_metadata(&provider, &subscriber);
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.name, name);
        assert_eq!(metadata.description, description);
    }

    #[test]
    fn test_subscription_not_found() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SoroSubsContract);
        let client = SoroSubsContractClient::new(&env, &contract_id);

        let provider = Address::generate(&env);
        let subscriber = Address::generate(&env);

        let sub = client.get_subscription(&provider, &subscriber);
        assert!(sub.is_none());

        let time_until = client.time_until_payment(&provider, &subscriber);
        assert_eq!(time_until, -1);

        assert!(!client.is_payment_due(&provider, &subscriber));
    }
}
