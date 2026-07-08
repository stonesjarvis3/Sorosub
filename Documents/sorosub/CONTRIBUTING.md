# Contributing to SoroSubs

Thank you for your interest in contributing to SoroSubs! This guide will help you understand how to contribute effectively to the project.

## Code of Conduct

Please read and follow our [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). We are committed to providing a welcoming and inclusive community for all contributors.

## Getting Started

### 1. Set Up Your Environment

```bash
# Clone the repository
git clone https://github.com/stonesjarvis3/Sorosub.git
cd Sorosub

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install Soroban CLI
cargo install stellar-cli

# Verify setup
cargo build --target wasm32-unknown-unknown --release
```

### 2. Explore the Wave Program

Review [WAVE_PROGRAM_PLAN.md](WAVE_PROGRAM_PLAN.md) to understand:
- Contribution categories (bug fixes, features, docs, testing, DevOps)
- Issue scoping guidelines
- Skill level recommendations
- Sprint structure

### 3. Find an Issue to Work On

1. Look for issues labeled:
   - `wave-program` - Wave Program eligible
   - `good-first-issue` - Beginner friendly
   - `help-wanted` - Community contributions welcome

2. Check the issue description for:
   - Acceptance criteria
   - Expected outcome
   - Implementation suggestions
   - Related documentation

3. Comment on the issue to claim it or ask questions

## Types of Contributions

### Bug Fixes

**Good fit for:**
- Experienced developers with contract knowledge
- Issues with clear reproduction steps

**Process:**
1. Read the bug report thoroughly
2. Create a test case that reproduces the issue
3. Fix the bug in minimal code changes
4. Verify the test now passes
5. Submit PR with clear explanation

**Example:**
```
Issue: Double-processing occurs when same payment called twice
- Add test case: test_double_processing_prevention
- Fix: Check LastProcessed timestamp before processing
- Verify: Test passes and no double-processing occurs
```

### New Features

**Good fit for:**
- Intermediate to advanced developers
- Features aligned with roadmap

**Process:**
1. Discuss design in the issue first
2. Get maintainer approval before starting
3. Implement with tests
4. Update relevant documentation
5. Submit PR with explanation

**Example:**
```
Feature: Multi-tier subscriptions
- Design: Separate subscription tiers (Bronze/Silver/Gold)
- Implementation: Add tier support to core contract
- Tests: Test each tier level and transitions
- Docs: Update README with tier examples
```

### Documentation

**Good fit for:**
- Writers, developers, and subject matter experts
- No coding required (usually)

**Types:**
- **Tutorials**: Step-by-step guides for specific tasks
- **API Docs**: Function documentation with examples
- **Guides**: How-to documents and best practices
- **Examples**: Code examples in multiple languages
- **Videos**: Tutorial or demonstration videos

**Process:**
1. Create or improve documentation
2. Test all code examples
3. Have someone else review for clarity
4. Submit PR with improvements

**Example:**
```
Tutorial: "Deploy SoroSubs to Production"
- Prerequisites and setup
- Step-by-step deployment
- Configuration options
- Troubleshooting section
- Verification checklist
```

### Testing

**Good fit for:**
- QA engineers, developers
- Improving test coverage

**Types:**
- **Unit Tests**: Test individual functions
- **Integration Tests**: Test function interactions
- **Edge Cases**: Test boundary conditions
- **Performance Tests**: Benchmark operations
- **Security Tests**: Test vulnerability scenarios

**Process:**
1. Identify test gaps
2. Write comprehensive tests
3. Ensure tests pass
4. Document test scenarios
5. Submit PR

**Example:**
```
Test: "Subscription modification edge cases"
- Test modify with max amount
- Test modify with min period
- Test modify inactive subscription (should fail)
- Test modify after cancellation
```

### DevOps & Infrastructure

**Good fit for:**
- DevOps engineers, infrastructure specialists

**Types:**
- **Deployment**: Docker, Kubernetes, CI/CD
- **Monitoring**: Dashboards, logging, alerts
- **Performance**: Profiling, optimization
- **Security**: Scanning, hardening

**Process:**
1. Propose infrastructure improvement
2. Design solution
3. Test in development environment
4. Document setup and usage
5. Submit PR

**Example:**
```
DevOps: "Add GitHub Actions CI/CD"
- Lint and format checking
- Test execution on push
- Build WASM binary verification
- Security scanning
```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout main
git pull origin main
git checkout -b feature/your-feature-name
```

Use descriptive branch names:
- `feature/multi-tier-subscriptions`
- `fix/double-processing-bug`
- `docs/react-integration-guide`
- `test/payment-edge-cases`

### 2. Make Your Changes

```bash
# For code changes
cd contracts/sorosub
cargo build --target wasm32-unknown-unknown --release
cargo test

# For documentation
# Edit .md files and verify formatting
```

### 3. Commit Your Changes

```bash
git add .
git commit -m "Clear, descriptive commit message"
```

Commit message format:
```
type: brief description

Longer explanation of changes if needed.

- Detail 1
- Detail 2

Fixes #123 (if applicable)
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

### 4. Push and Create a Pull Request

```bash
git push origin feature/your-feature-name
```

Then:
1. Go to GitHub and create a PR
2. Fill out the PR template completely
3. Link related issues
4. Request reviewers

### 5. Address Review Feedback

```bash
# Make changes based on feedback
git add .
git commit -m "Address review feedback"
git push origin feature/your-feature-name
```

The PR will update automatically.

### 6. Merge

After approval, maintainers will merge your PR. You can then delete your branch:

```bash
git checkout main
git pull origin main
git branch -d feature/your-feature-name
```

## Pull Request Guidelines

### Before Submitting

- [ ] Code follows project style guidelines
- [ ] Tests added/updated and passing
- [ ] Documentation updated
- [ ] No compiler warnings
- [ ] Commit messages are clear
- [ ] Branch is up to date with main

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation
- [ ] Testing
- [ ] DevOps

## Related Issues
Fixes #123

## Changes Made
- Change 1
- Change 2

## Testing Done
- Test 1
- Test 2

## Screenshots (if applicable)
[Add screenshots for UI/doc changes]

## Checklist
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] No breaking changes
- [ ] Ready for review
```

## Code Style Guidelines

### Rust Code

- Follow Rust conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Add comments for complex logic
- Document public functions

```rust
/// Processes a due payment between provider and subscriber
/// 
/// # Arguments
/// * `env` - Soroban environment
/// * `provider` - Payment recipient
/// * `subscriber` - Payment payer
///
/// # Panics
/// If subscription not found or payment not due
pub fn process_payment(env: Env, provider: Address, subscriber: Address) {
    // Implementation
}
```

### Documentation

- Use clear, simple language
- Include code examples
- Add helpful context
- Update table of contents if needed
- Test all code examples

### Commit Messages

- Start with type: `feat:`, `fix:`, `docs:`
- Use imperative mood ("Add feature" not "Added feature")
- Keep first line under 70 characters
- Reference issues with `Fixes #123`

## Testing Requirements

### For Code Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### For Documentation

- Verify all links work
- Test all code examples
- Check formatting and spacing
- Ensure clarity for beginners

## Performance Considerations

- Changes should maintain O(1) performance
- Monitor gas costs for contract changes
- Benchmark before and after
- Document performance impact

## Security Considerations

- Follow secure coding practices
- Validate all inputs
- Handle errors gracefully
- Never store secrets in code
- Review security implications

## Getting Help

- **Questions**: Ask in GitHub Discussions
- **Issues**: Check existing issues first
- **Documentation**: Read [INDEX.md](INDEX.md)
- **Contact**: Email contributors@sorosub.example

## Recognition

Contributors are recognized through:
- Acknowledgment in release notes
- Addition to CONTRIBUTORS.md
- Community highlights
- Speaking opportunities
- Leadership roles

## Review Process

Maintainers will:
- Review PRs within 7 days
- Provide constructive feedback
- Request changes if needed
- Merge once approved
- Thank you for contributing!

## License

By contributing to SoroSubs, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to SoroSubs!** 🎉

Your contributions help make recurring payments on Stellar accessible and reliable for everyone.