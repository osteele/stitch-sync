# Security Maintenance Guide

## Addressing Dependency Vulnerabilities

GitHub has detected vulnerabilities in the project dependencies. This guide explains how to address them.

## Quick Steps

```bash
# 1. Install cargo-audit (one-time setup)
cargo install cargo-audit

# 2. Scan for vulnerabilities
cargo audit

# 3. Update dependencies
cargo update

# 4. Check for outdated dependencies
cargo install cargo-outdated
cargo outdated

# 5. Run tests to ensure compatibility
cargo test

# 6. Commit the updated Cargo.lock
git add Cargo.lock
git commit -m "chore: update dependencies to address security vulnerabilities"
```

## Detailed Vulnerability Resolution

### Step 1: Identify Vulnerabilities

```bash
cargo audit
```

This will show:
- Which dependencies have known vulnerabilities
- Severity levels (low, moderate, high, critical)
- CVE numbers
- Recommended versions

### Step 2: Update Dependencies

There are two approaches:

#### Option A: Conservative Update (Recommended)
```bash
# Updates to latest compatible versions within Cargo.toml constraints
cargo update
```

This respects version constraints in `Cargo.toml` (e.g., `"4.4"` means `>=4.4, <5.0`)

#### Option B: Major Version Updates
If `cargo update` doesn't fix the vulnerability:

1. Check the audit output for recommended versions
2. Update `Cargo.toml` with new version constraints:
   ```toml
   # Example: Update clap from 4.4 to 4.5
   clap = { version = "4.5", features = ["derive"] }
   ```
3. Run `cargo update`
4. Test thoroughly for breaking changes

### Step 3: Verify Fixes

```bash
# Re-run audit to confirm vulnerabilities are resolved
cargo audit

# Run all tests
cargo test

# Try building in release mode
cargo build --release
```

### Step 4: Test Critical Functionality

Manually test:
- [ ] File watching works
- [ ] File conversion with Inkscape
- [ ] USB drive detection on your platform
- [ ] Configuration management
- [ ] Machine database queries

### Step 5: Commit Changes

```bash
git add Cargo.lock Cargo.toml  # If Cargo.toml was modified
git commit -m "fix: update dependencies to address security vulnerabilities

Resolves: [vulnerability details]
- Updated [package] from x.y.z to a.b.c
- Fixes CVE-XXXX-XXXXX"

git push
```

## Common Vulnerabilities in Dependencies

### Windows Crate

The `windows` crate is frequently updated for security:
- **Current version in project**: Check `Cargo.toml` (0.48)
- **Latest stable**: Usually several versions ahead
- **Impact**: Affects Windows platform only

**Resolution**:
```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [...] }  # Update version
```

### Reqwest

Used for HTTP requests (self-update feature):
- **Security concerns**: HTTP client vulnerabilities, TLS issues
- **Impact**: Network operations

**Resolution**:
```bash
cargo update -p reqwest
```

### Other Common Issues

1. **Time/Chrono**: Often has RUSTSEC advisories
2. **Tokio/Async**: Can have race condition vulnerabilities
3. **Serialization crates**: JSON/YAML parsing vulnerabilities

## Automated Dependency Updates

### Setup Dependabot

The project should have `.github/dependabot.yml`:

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10
    reviewers:
      - "osteele"
    labels:
      - "dependencies"
      - "security"
```

This will:
- Check for updates weekly
- Create PRs for vulnerable dependencies
- Group related updates together

### Review Dependabot PRs

When Dependabot creates a PR:

1. **Check the changelog**: What changed in the update?
2. **Review breaking changes**: Are there API changes?
3. **Run CI**: Ensure all tests pass
4. **Merge promptly**: Security updates should be merged quickly

## Monitoring for New Vulnerabilities

### GitHub Security Alerts

- Enable security alerts in repository settings
- Check: https://github.com/osteele/stitch-sync/security/dependabot
- Review weekly

### RustSec Advisory Database

- Visit: https://rustsec.org/
- Subscribe to: https://rustsec.org/feed.xml
- Check before releases

## Version Update Strategy

### Patch Updates (0.1.x → 0.1.y)
- **Always safe**: Bug fixes and security patches
- **Action**: Merge immediately

### Minor Updates (0.x.0 → 0.y.0)
- **Usually safe**: New features, backward compatible
- **Action**: Review changes, test, merge within 1 week

### Major Updates (x.0.0 → y.0.0)
- **Breaking changes**: API changes possible
- **Action**: Review carefully, plan migration, test thoroughly

## Testing After Updates

```bash
# Run full test suite
cargo test --all-features

# Test on all platforms (use CI or local VMs)
cargo test --target x86_64-unknown-linux-gnu
cargo test --target x86_64-pc-windows-msvc
cargo test --target x86_64-apple-darwin

# Check for compilation issues
cargo clippy --all-targets --all-features

# Verify formatting
cargo fmt --check
```

## Rollback Strategy

If an update causes issues:

```bash
# 1. Revert Cargo.lock
git checkout HEAD~1 Cargo.lock

# 2. If Cargo.toml was changed, revert it too
git checkout HEAD~1 Cargo.toml

# 3. Rebuild
cargo build

# 4. Create an issue to track the problem
# 5. Investigate why the update failed
# 6. Try a different version
```

## Current Known Issues

As of the last check, GitHub reported:
- **1 high severity** vulnerability
- **3 moderate severity** vulnerabilities
- **1 low severity** vulnerability

To see current status:
```bash
# Visit Dependabot alerts
open https://github.com/osteele/stitch-sync/security/dependabot

# Or use GitHub CLI
gh api repos/osteele/stitch-sync/dependabot/alerts
```

## Prevention

### Before Adding New Dependencies

1. Check popularity and maintenance status
2. Review recent issues and PRs
3. Check for known vulnerabilities: `cargo audit`
4. Consider alternatives with better security records

### Regular Maintenance

- **Weekly**: Review Dependabot alerts
- **Monthly**: Run `cargo update` and `cargo outdated`
- **Before releases**: Run `cargo audit`
- **Quarterly**: Review all dependencies for alternatives

## Resources

- **RustSec Database**: https://rustsec.org/
- **Cargo Audit**: https://docs.rs/cargo-audit/
- **Dependabot**: https://docs.github.com/en/code-security/dependabot
- **Cargo Update**: https://doc.rust-lang.org/cargo/commands/cargo-update.html

## Support

For questions about security updates:
- Create an issue: https://github.com/osteele/stitch-sync/issues
- Email: steele@osteele.com
