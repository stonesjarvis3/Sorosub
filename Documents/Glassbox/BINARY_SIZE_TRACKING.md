# Binary Size Tracking

This document describes the binary size tracking system implemented for the Glassbox project to detect and prevent binary size regressions.

## Overview

The binary size tracking system monitors the size of compiled artifacts (CLI binaries and library files) and compares them against configured thresholds. This helps:

- **Detect regressions**: Catch unexpected size increases early in development
- **Maintain performance**: Keep binaries lean and fast to download/deploy
- **Track trends**: Monitor size changes over time
- **Enforce limits**: Prevent binaries from growing beyond acceptable limits

## Components

### 1. Configuration File: `size-thresholds.json`

Defines size limits for each binary artifact:

```json
{
  "version": "1.0.0",
  "thresholds": {
    "glassbox-cli": {
      "max_size_bytes": 15728640,
      "max_size_human": "15 MB",
      "warn_threshold_percent": 90,
      "description": "Main CLI binary for Glassbox trace viewer"
    }
  }
}
```

**Fields:**
- `max_size_bytes`: Hard limit in bytes - builds fail if exceeded
- `max_size_human`: Human-readable representation of the limit
- `warn_threshold_percent`: Percentage of max_size at which warnings are issued
- `description`: Purpose of the binary

### 2. Size Check Script: `scripts/check-binary-sizes.sh`

Main script that measures binary sizes and compares them against thresholds.

**Usage:**
```bash
./scripts/check-binary-sizes.sh
```

**Environment Variables:**
- `SIZE_CONFIG_FILE`: Path to configuration file (default: `size-thresholds.json`)
- `TARGET_DIR`: Directory containing built binaries (default: `target/release`)
- `OUTPUT_FILE`: Path for JSON report output (default: `size-report.json`)

**Exit Codes:**
- `0`: All checks passed (may include warnings)
- `1`: One or more checks failed

**Output:**
- Console output with colored status indicators
- JSON report file (`size-report.json`) with detailed results

### 3. Baseline Measurement Script: `scripts/measure-baseline-sizes.sh`

Measures current binary sizes and optionally updates thresholds.

**Usage:**
```bash
# Measure current sizes
./scripts/measure-baseline-sizes.sh

# Measure and update thresholds (adds 20% buffer)
./scripts/measure-baseline-sizes.sh --update-config
```

**When to use:**
- After legitimate size increases (new features, dependencies)
- When establishing initial baselines
- After optimization work to tighten thresholds

### 4. GitHub Actions Workflow: `.github/workflows/size-check.yml`

Automated CI pipeline that runs size checks on every push and pull request.

**Features:**
- Builds release binaries with caching
- Runs size checks automatically
- Uploads size reports as artifacts (90-day retention)
- Posts results to PR comments
- Adds summary to GitHub Actions UI
- Fails the build if thresholds are exceeded

### 5. Test Suite: `scripts/test-size-tracking.sh`

Validates the size tracking system itself.

**Usage:**
```bash
./scripts/test-size-tracking.sh
```

**Tests:**
- Configuration file validity
- Script existence and permissions
- Workflow file presence
- Threshold value sanity checks
- Runtime behavior (if binaries are built)

## Workflow

### For Developers

1. **Make changes** to the codebase
2. **Build release binaries**: `cargo build --release`
3. **Check sizes locally**: `./scripts/check-binary-sizes.sh`
4. **Review results**: Check console output and `size-report.json`
5. **If size increased significantly**:
   - Investigate the cause
   - Optimize if possible
   - Update thresholds if increase is justified

### In CI/CD

1. **Push or create PR** triggers the workflow
2. **Workflow builds** release binaries
3. **Size check runs** automatically
4. **Results posted** to PR comments and job summary
5. **Build fails** if thresholds exceeded
6. **Size report** uploaded as artifact for historical tracking

## Thresholds

### Current Thresholds

| Binary | Max Size | Warn At | Description |
|--------|----------|---------|-------------|
| glassbox-cli | 15 MB | 13.5 MB (90%) | Main CLI binary |
| watch-demo | 15 MB | 13.5 MB (90%) | Watch mode demo |
| libglassbox.rlib | 10 MB | 9 MB (90%) | Library artifact |

### Threshold Philosophy

- **Max size**: Set 20% above current baseline to allow for reasonable growth
- **Warn threshold**: Set at 90% to provide early warning before hitting limit
- **Review regularly**: Update thresholds as project evolves

### When to Update Thresholds

**Increase thresholds when:**
- Adding significant new features
- Adding new dependencies
- Legitimate architectural changes

**Decrease thresholds when:**
- Removing features or dependencies
- After optimization work
- Switching to lighter alternatives

**Process:**
1. Build optimized release binaries
2. Run: `./scripts/measure-baseline-sizes.sh --update-config`
3. Review the changes in `size-thresholds.json`
4. Commit the updated configuration
5. Document the reason in the commit message

## Size Report Format

The `size-report.json` file contains:

```json
{
  "timestamp": "2026-06-01T12:00:00Z",
  "summary": {
    "total": 3,
    "passed": 2,
    "warnings": 1,
    "failures": 0
  },
  "results": [
    {
      "name": "glassbox-cli",
      "path": "target/release/glassbox-cli",
      "actual_size": 12582912,
      "max_size": 15728640,
      "percent": "80.0",
      "status": "PASS",
      "description": "Main CLI binary"
    }
  ]
}
```

## Troubleshooting

### Size Check Fails Locally

1. **Check if binaries are built**: `ls -lh target/release/`
2. **Build release binaries**: `cargo build --release`
3. **Run size check**: `./scripts/check-binary-sizes.sh`
4. **Review output**: Look for which binary exceeded threshold

### Size Check Fails in CI

1. **Check the workflow logs** for detailed output
2. **Download the size report artifact** from the workflow run
3. **Compare with previous runs** to identify the change
4. **Review recent commits** that might have increased size

### jq Not Found Error

Install jq:
- **Ubuntu/Debian**: `sudo apt-get install jq`
- **macOS**: `brew install jq`
- **Fedora**: `sudo dnf install jq`

### Permission Denied on Scripts

Make scripts executable:
```bash
chmod +x scripts/*.sh
```

## Best Practices

### 1. Check Sizes Before Committing

```bash
cargo build --release && ./scripts/check-binary-sizes.sh
```

### 2. Monitor Size Trends

- Review size reports in CI artifacts
- Track size changes in PR comments
- Investigate unexpected increases

### 3. Optimize When Possible

- Use `cargo bloat` to identify large dependencies
- Enable link-time optimization (LTO) in release builds
- Consider feature flags to make dependencies optional
- Strip debug symbols in production builds

### 4. Document Size Changes

When updating thresholds, include in commit message:
- Why the size increased
- What was added/changed
- Whether optimization was attempted
- New threshold values

### 5. Regular Reviews

- Review thresholds quarterly
- Update baselines after major releases
- Tighten thresholds after optimization work

## Integration with Development Workflow

### Pre-commit Hook (Optional)

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
if [ -f "target/release/glassbox-cli" ]; then
    ./scripts/check-binary-sizes.sh || {
        echo "Warning: Binary size check failed"
        echo "Run './scripts/check-binary-sizes.sh' for details"
    }
fi
```

### Makefile Integration

Add to `Makefile`:

```makefile
.PHONY: size-check
size-check:
	cargo build --release
	./scripts/check-binary-sizes.sh

.PHONY: update-size-baselines
update-size-baselines:
	cargo build --release
	./scripts/measure-baseline-sizes.sh --update-config
```

## Future Enhancements

Potential improvements to consider:

1. **Historical tracking**: Store size history in a database or file
2. **Trend analysis**: Generate charts showing size over time
3. **Per-dependency analysis**: Break down size by dependency
4. **Automatic optimization**: Suggest optimizations when size increases
5. **Multiple profiles**: Different thresholds for debug vs release builds
6. **Size budgets**: Allocate size budgets to different modules

## References

- [Cargo Book - Build Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat) - Find what takes space in your binary
- [cargo-strip](https://doc.rust-lang.org/cargo/reference/manifest.html#the-strip-field) - Strip symbols from binaries
- [GitHub Actions - Artifacts](https://docs.github.com/en/actions/using-workflows/storing-workflow-data-as-artifacts)

## Support

For issues or questions about binary size tracking:

1. Check this documentation
2. Review the test suite: `./scripts/test-size-tracking.sh`
3. Examine size reports in CI artifacts
4. Open an issue with the `size-tracking` label

---

**Related Issues:**
- Closes #134: Add binary size tracking to build pipeline
