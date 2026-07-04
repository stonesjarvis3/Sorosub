# Security Policy

## Overview

SoroSubs is a smart contract for recurring payments on the Stellar Soroban platform. This document outlines security considerations, potential risks, and recommended practices.

## Security Model

### Authentication
- All sensitive operations require explicit caller authentication via `require_auth()`
- Subscription creation and modification require the subscriber's signature
- Payment processing is stateless and requires no specific authorization
- Provider modifications require provider authorization

### Authorization
- Only the subscriber can authorize subscription creation
- Only the subscriber can cancel a subscription
- Only the provider or subscriber can modify subscription terms
- Any address can trigger payment processing (stateless operation)

### Data Integrity
- Subscriptions are immutable once stored (replaced on modification)
- Timestamps are sourced from Soroban ledger (cannot be manipulated by users)
- All state changes emit events for off-chain verification

## Vulnerability Analysis

### Known Protections

#### Double-Processing Prevention
```rust
// Check for spam/abuse - prevent processing same subscription twice in same ledger
let last_processed_key = DataKey::LastProcessed(provider.clone(), subscriber.clone());
if let Some(last_timestamp) = env.storage().persistent().get::<_, u64>(&last_processed_key) {
    if last_timestamp == now {
        panic!("Payment already processed in this block");
    }
}
```
This prevents the same subscription from being processed multiple times in a single block.

#### Input Validation
- Amount must be between 1 and 1 billion USDC
- Period must be between 60 seconds and ~10 years
- Provider and subscriber must be different
- Subscriptions cannot be created if one already exists

#### Reentrancy Safety
- Contract never calls external code (only token transfer interface)
- No callbacks or hooks that could be exploited
- State changes happen after all external calls

### Potential Risks & Mitigations

#### Token Transfer Failure
**Risk**: A payment might fail if subscriber lacks balance or allowance.
**Mitigation**: Subscription remains unchanged; caller can retry. Consider monitoring via events.
**Recommendation**: Off-chain monitoring should track failed payments and alert users.

#### Quantum Computing / Signature Forgery
**Risk**: Stellar's cryptography could theoretically be broken.
**Mitigation**: This is an ecosystem-level concern, not contract-specific.
**Recommendation**: Follow Stellar's quantum security roadmap.

#### Time Manipulation
**Risk**: In a controlled environment, ledger timestamps could theoretically be adjusted.
**Mitigation**: This requires consensus-level control; not a contract vulnerability.
**Recommendation**: Trust the validator network's time consensus.

#### Admin Address Compromise
**Risk**: If admin key is compromised, no direct impact (admin has no active privileges).
**Mitigation**: Current implementation stores admin address but doesn't use it.
**Recommendation**: Future emergency pause functionality should use multi-sig.

#### Subscription Enumeration
**Risk**: User activity could be tracked via on-chain events.
**Mitigation**: This is inherent to blockchain; metadata can be hashed if privacy needed.
**Recommendation**: For privacy-sensitive use cases, consider encryption layers.

## Audit Recommendations

### Off-Chain Components
1. **Payment Processor**: Service that calls `process_payment` should have:
   - Rate limiting (prevent spam)
   - Error handling and retries
   - Monitoring and alerting
   - Access logs

2. **Token Integration**: Verify:
   - USDC contract is on-chain verified
   - No fake token addresses accepted
   - Allowance checks before processing

3. **Key Management**:
   - Multi-sig for admin functions (if added)
   - Hardware wallets for high-value subscriptions
   - Regular key rotation policy

### On-Chain Verification
1. Deploy to testnet and test:
   - All state transitions
   - Event emissions
   - Edge cases (max/min values, etc.)
   - Multiple concurrent subscriptions

2. Run formal verification tools if available for Soroban

3. Test recovery scenarios:
   - What happens if payment processor crashes?
   - Can payments be skipped?
   - Are state machines always consistent?

## Best Practices for Users

### Subscribers
1. **Verify Provider Address**: Double-check provider address before subscribing
2. **Start Small**: Test with small amounts before large subscriptions
3. **Monitor Payments**: Verify payments go through as expected
4. **Secure Keys**: Use hardware wallets or secure key management
5. **Review Terms**: Understand the payment amount and frequency

### Providers
1. **Monitor Incoming Payments**: Track subscription revenue
2. **Document Terms**: Keep records of subscription agreements (off-chain)
3. **Communicate Changes**: Notify subscribers before modifying payment terms
4. **Plan for Cancellations**: Handle when subscribers cancel

### Operators
1. **Reliable Payment Processing**: Ensure `process_payment` is called reliably
2. **Handle Failures**: Implement retry logic with exponential backoff
3. **Monitor Events**: Index and monitor all contract events
4. **Test Regularly**: Periodically test payment processing on testnet

## Incident Response

### If a Payment Fails
1. Check subscriber balance and allowance
2. Review event logs for the subscription
3. Call `time_until_payment` to verify timing
4. Retry `process_payment` (idempotent due to `LastProcessed` check)

### If a Subscription Gets Stuck
1. Verify subscription exists via `get_subscription`
2. Check if it's marked inactive
3. Cancel and recreate if necessary

### If You Suspect a Bug
1. Do NOT use the contract with high-value subscriptions
2. Deploy a new version to testnet
3. Isolate the issue with minimal test case
4. Report to security@sorosub.example (if available)

## Deployment Checklist

Before deploying to mainnet:

- [ ] Code reviewed by 2+ independent developers
- [ ] All tests passing locally
- [ ] Deployed to testnet with real transactions
- [ ] Monitored on testnet for 7+ days
- [ ] Event system verified working
- [ ] Payment processor tested end-to-end
- [ ] Documentation updated
- [ ] Emergency pause procedure documented (if not implemented)
- [ ] Backup/recovery plan in place
- [ ] Legal review (if required by jurisdiction)

## Contact & Disclosure

For security vulnerabilities:
1. **Do NOT** open public issues
2. Email: security@sorosub.example (replace with actual contact)
3. Include: Description, reproduction steps, potential impact
4. Allow 30 days for patch before disclosure

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024 | Initial release |
