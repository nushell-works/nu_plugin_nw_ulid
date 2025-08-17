# Release Procedures and Versioning Strategy

**Version**: 1.0  
**Last Updated**: August 18, 2025  
**Target Audience**: Maintainers and release managers

This document outlines the comprehensive release procedures, versioning strategy, and quality gates for nu_plugin_ulid releases.

## Table of Contents

1. [Versioning Strategy](#versioning-strategy)
2. [Release Types](#release-types)
3. [Release Process](#release-process)
4. [Quality Gates](#quality-gates)
5. [Automated Release Pipeline](#automated-release-pipeline)
6. [Manual Release Steps](#manual-release-steps)
7. [Post-Release Procedures](#post-release-procedures)
8. [Emergency Releases](#emergency-releases)
9. [Release Communication](#release-communication)

## Versioning Strategy

### Semantic Versioning

nu_plugin_ulid follows [Semantic Versioning 2.0.0](https://semver.org/) with the format `MAJOR.MINOR.PATCH`:

- **MAJOR**: Incompatible API changes or breaking changes
- **MINOR**: Backwards-compatible new features and functionality
- **PATCH**: Backwards-compatible bug fixes and improvements

### Version Examples

```
0.1.0   - Initial development release
0.2.0   - Added new commands (minor)
0.2.1   - Bug fixes (patch)
1.0.0   - First stable release (major)
1.1.0   - New features (minor)
1.1.1   - Bug fixes (patch)
2.0.0   - Breaking changes (major)
```

### Pre-Release Versions

For pre-release versions, we use the following suffixes:

- **Alpha**: `1.0.0-alpha.1` - Early development, unstable
- **Beta**: `1.0.0-beta.1` - Feature complete, testing phase
- **Release Candidate**: `1.0.0-rc.1` - Stable, final testing

### Branch Strategy

- **main**: Stable, release-ready code
- **develop**: Integration branch for new features
- **feature/***: Individual feature development
- **release/***: Release preparation branches
- **hotfix/***: Critical bug fixes for production

## Release Types

### Regular Releases

#### Minor Releases (Monthly)
- **Schedule**: First Tuesday of each month
- **Content**: New features, improvements, non-breaking changes
- **Testing**: Full test suite + integration testing
- **Review**: Community review period (1 week)

#### Patch Releases (As Needed)
- **Schedule**: As needed for bug fixes
- **Content**: Bug fixes, security patches, minor improvements
- **Testing**: Focused testing on changed areas
- **Review**: Maintainer review only

#### Major Releases (Quarterly)
- **Schedule**: Quarterly or when breaking changes accumulate
- **Content**: Breaking changes, major new features, architectural changes
- **Testing**: Comprehensive testing + performance validation
- **Review**: Extended community review (2 weeks)

### Special Releases

#### Security Releases
- **Schedule**: Immediate upon security issue discovery
- **Content**: Security fixes only
- **Testing**: Security validation + regression testing
- **Review**: Security team review

#### LTS Releases
- **Schedule**: Annually for major versions
- **Content**: Stable, long-term supported versions
- **Support**: Extended support and maintenance (2 years)
- **Testing**: Extensive compatibility and stability testing

## Release Process

### Phase 1: Preparation (1-2 weeks before release)

#### 1.1 Release Planning
```bash
# Create release branch
git checkout -b release/v1.2.0
git push -u origin release/v1.2.0
```

#### 1.2 Version Updates
```bash
# Update version in Cargo.toml
[package]
version = "1.2.0"

# Update version in documentation
docs/USER_GUIDE.md
docs/DEVELOPER_GUIDE.md
README.md
```

#### 1.3 Documentation Updates
- [ ] Update CHANGELOG.md with release notes
- [ ] Review and update all documentation
- [ ] Verify examples and tutorials
- [ ] Update API documentation
- [ ] Validate migration guides

#### 1.4 Dependency Management
```bash
# Update dependencies
cargo update

# Security audit
cargo audit

# Dependency review
cargo deny check
```

### Phase 2: Testing and Validation (1 week)

#### 2.1 Automated Testing
```bash
# Run full test suite
cargo test --all-features --release

# Run security tests
cargo test --test security_tests --release

# Run performance tests
cargo test --test performance_tests --release

# Run integration tests
cargo test --test integration_tests --release
```

#### 2.2 Cross-Platform Testing
```bash
# Test on all supported platforms
# - Linux (x86_64, aarch64)
# - macOS (x86_64, aarch64)
# - Windows (x86_64)

# CI/CD pipeline handles this automatically
```

#### 2.3 Performance Validation
```bash
# Run benchmarks
cargo bench

# Compare with previous version
cargo bench -- --save-baseline previous
cargo bench -- --baseline previous
```

#### 2.4 Compatibility Testing
```bash
# Test with different Nushell versions
# - Minimum supported version
# - Latest stable version
# - Latest development version
```

### Phase 3: Release Candidate (3-5 days)

#### 3.1 Create Release Candidate
```bash
# Tag release candidate
git tag v1.2.0-rc.1
git push origin v1.2.0-rc.1

# Trigger RC build
gh workflow run release.yml --ref v1.2.0-rc.1
```

#### 3.2 Community Testing
- [ ] Announce RC to community
- [ ] Collect feedback and testing results
- [ ] Address critical issues
- [ ] Update documentation based on feedback

#### 3.3 Final Validation
- [ ] All automated tests pass
- [ ] Manual testing completed
- [ ] Documentation reviewed
- [ ] Security audit completed
- [ ] Performance benchmarks acceptable

### Phase 4: Release (Release day)

#### 4.1 Final Preparations
```bash
# Final version update
# Remove -rc suffix
# Final commit and push

# Merge to main
git checkout main
git merge --no-ff release/v1.2.0
```

#### 4.2 Create Release
```bash
# Create release tag
git tag v1.2.0
git push origin v1.2.0

# Trigger release pipeline
gh workflow run release.yml --ref v1.2.0
```

#### 4.3 Automated Release Steps
The automated pipeline handles:
- [ ] Multi-platform binary builds
- [ ] Crates.io publication
- [ ] GitHub release creation
- [ ] Documentation deployment
- [ ] Release announcement

## Quality Gates

### Mandatory Quality Gates

All releases must pass these quality gates:

#### ✅ Code Quality
- [ ] Zero clippy warnings
- [ ] All tests pass (100% success rate)
- [ ] Code coverage ≥ 90%
- [ ] No security vulnerabilities
- [ ] Dependency audit clean

#### ✅ Performance
- [ ] No performance regressions ≥ 5%
- [ ] Memory usage within acceptable bounds
- [ ] Benchmark suite passes
- [ ] Load testing for critical paths

#### ✅ Security
- [ ] Security audit completed
- [ ] No critical or high severity vulnerabilities
- [ ] Dependency security scan clean
- [ ] Security tests pass

#### ✅ Documentation
- [ ] All user-facing changes documented
- [ ] API documentation complete
- [ ] Examples tested and verified
- [ ] Migration guides provided (for breaking changes)

#### ✅ Compatibility
- [ ] Minimum Rust version supported
- [ ] Nushell compatibility maintained
- [ ] Cross-platform compatibility verified
- [ ] Backwards compatibility preserved (non-major releases)

### Quality Gate Checklist

```bash
# Run complete quality gate validation
./scripts/quality_gates.sh

# This script runs:
# - cargo test --all-features
# - cargo clippy -- -D warnings
# - cargo audit
# - cargo deny check
# - Security tests
# - Performance benchmarks
# - Cross-platform builds
```

## Automated Release Pipeline

### GitHub Actions Workflow

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  quality-gates:
    runs-on: ubuntu-latest
    steps:
      - name: Run Quality Gates
        run: ./scripts/quality_gates.sh

  build-binaries:
    needs: quality-gates
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        arch: [x86_64, aarch64]
    runs-on: ${{ matrix.os }}
    steps:
      - name: Build Release Binary
        run: cargo build --release --target ${{ matrix.arch }}

  publish-crates:
    needs: [quality-gates, build-binaries]
    runs-on: ubuntu-latest
    steps:
      - name: Publish to Crates.io
        run: cargo publish --token ${{ secrets.CRATES_TOKEN }}

  create-release:
    needs: [publish-crates]
    runs-on: ubuntu-latest
    steps:
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: target/release/nu_plugin_ulid*
          generate_release_notes: true
```

### Release Automation Scripts

#### Quality Gates Script
```bash
#!/bin/bash
# scripts/quality_gates.sh

set -e

echo "🔍 Running Quality Gates..."

# Code quality
echo "📋 Checking code quality..."
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check

# Testing
echo "🧪 Running tests..."
cargo test --all-features --release

# Security
echo "🔒 Security audit..."
cargo audit
cargo deny check

# Performance
echo "⚡ Performance benchmarks..."
cargo bench --quiet

# Documentation
echo "📚 Documentation check..."
cargo doc --all-features --no-deps

echo "✅ All quality gates passed!"
```

#### Release Script
```bash
#!/bin/bash
# scripts/release.sh

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "🚀 Preparing release $VERSION..."

# Update version
sed -i "s/version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update changelog
echo "Updating CHANGELOG.md..."
# Add changelog entry

# Commit and tag
git add .
git commit -m "chore: prepare release v$VERSION"
git tag "v$VERSION"

echo "✅ Release v$VERSION prepared!"
echo "🔄 Push with: git push origin main && git push origin v$VERSION"
```

## Manual Release Steps

### Pre-Release Checklist

- [ ] **Version Planning**: Determine version number based on changes
- [ ] **Branch Management**: Create and prepare release branch
- [ ] **Documentation**: Update all documentation and examples
- [ ] **Testing**: Run comprehensive test suite
- [ ] **Security**: Complete security audit and vulnerability scan
- [ ] **Performance**: Validate performance benchmarks
- [ ] **Dependencies**: Update and audit dependencies

### Release Day Checklist

- [ ] **Final Testing**: Run final test suite on release branch
- [ ] **Version Update**: Update version numbers in all files
- [ ] **Changelog**: Finalize changelog with release notes
- [ ] **Tag Creation**: Create and push release tag
- [ ] **Pipeline Trigger**: Monitor automated release pipeline
- [ ] **Verification**: Verify successful publication to Crates.io
- [ ] **GitHub Release**: Verify GitHub release creation
- [ ] **Documentation**: Verify documentation deployment

### Post-Release Checklist

- [ ] **Announcement**: Publish release announcement
- [ ] **Community**: Notify community channels
- [ ] **Monitoring**: Monitor for issues and feedback
- [ ] **Metrics**: Collect download and usage metrics
- [ ] **Planning**: Plan next release cycle

## Post-Release Procedures

### Release Monitoring

#### First 24 Hours
- [ ] Monitor GitHub issues for release-related problems
- [ ] Check Crates.io download statistics
- [ ] Monitor community channels for feedback
- [ ] Verify documentation is accessible
- [ ] Check automated tests on main branch

#### First Week
- [ ] Analyze user feedback and bug reports
- [ ] Plan patch release if critical issues found
- [ ] Update project roadmap based on feedback
- [ ] Collect performance metrics from users
- [ ] Update release process based on lessons learned

### Release Communication

#### Internal Communication
- [ ] Update maintainer team on release status
- [ ] Document any issues encountered
- [ ] Update release procedures based on experience
- [ ] Plan improvements for next release

#### External Communication
- [ ] Post release announcement on GitHub
- [ ] Update project website and documentation
- [ ] Notify Nushell community
- [ ] Share on relevant social media channels
- [ ] Update package manager listings

## Emergency Releases

### When to Create Emergency Releases

- **Critical Security Vulnerabilities**: Immediate threat to user security
- **Data Loss Bugs**: Bugs that could cause user data loss
- **Severe Regressions**: Critical functionality completely broken
- **Compliance Issues**: Legal or compliance-related urgent fixes

### Emergency Release Process

#### Immediate Response (0-2 hours)
1. **Assessment**: Confirm severity and impact
2. **Team Assembly**: Notify all maintainers
3. **Impact Analysis**: Assess affected users and systems
4. **Fix Development**: Begin immediate fix development

#### Rapid Development (2-8 hours)
1. **Hotfix Branch**: Create hotfix branch from latest release
2. **Minimal Fix**: Implement minimal fix for the issue
3. **Targeted Testing**: Test only the specific fix and regression
4. **Security Review**: Additional security review if applicable

#### Emergency Release (8-12 hours)
1. **Version Bump**: Increment patch version
2. **Minimal Documentation**: Update only critical documentation
3. **Expedited Review**: Fast-track review process
4. **Emergency Deploy**: Deploy using expedited pipeline
5. **Immediate Communication**: Notify users immediately

### Emergency Release Template

```markdown
# Emergency Release v1.2.1

## Critical Issue
Brief description of the critical issue that required emergency release.

## Impact
- Who is affected
- What functionality is impacted
- Severity level

## Fix
- What was changed
- Why this approach was chosen
- Testing performed

## Immediate Actions Required
- [ ] Update immediately
- [ ] Verify fix in your environment
- [ ] Report any continued issues

## Next Steps
- Full regression testing will be performed
- Comprehensive fix will be included in next regular release
- Post-mortem will be conducted
```

## Release Communication

### Release Announcement Template

```markdown
# nu_plugin_ulid v1.2.0 Released 🎉

We're excited to announce the release of nu_plugin_ulid v1.2.0! This release includes [brief summary of major changes].

## 🚀 What's New

### New Features
- Feature 1: Description and benefit
- Feature 2: Description and benefit

### Improvements
- Improvement 1: Performance increase of X%
- Improvement 2: Better error messages

### Bug Fixes
- Fix 1: Resolution of issue #123
- Fix 2: Resolution of issue #456

## 📈 Performance Improvements

- ULID generation: 15% faster
- Memory usage: 20% reduction
- Large dataset processing: 2x improvement

## 🔒 Security Updates

- Updated dependencies with security patches
- Enhanced input validation
- Improved error message sanitization

## 📋 Breaking Changes

[If any breaking changes, describe them here with migration instructions]

## 🛠️ Installation

```bash
cargo install nw-nu_plugin_ulid@1.2.0
```

## 📚 Documentation

- [User Guide](docs/USER_GUIDE.md)
- [Migration Guide](docs/MIGRATION_GUIDE.md)
- [Changelog](CHANGELOG.md)

## 🙏 Contributors

Special thanks to all contributors who made this release possible:
- @contributor1
- @contributor2

## 📞 Support

- [GitHub Issues](https://github.com/nushell-works/nu_plugin_ulid/issues)
- [Discussions](https://github.com/nushell-works/nu_plugin_ulid/discussions)
- [Documentation](https://github.com/nushell-works/nu_plugin_ulid/tree/main/docs)

Happy coding! 🦀
```

### Communication Channels

#### Primary Channels
- **GitHub Releases**: Automated release notes and downloads
- **Crates.io**: Package manager integration
- **Project Documentation**: Updated guides and references

#### Community Channels
- **GitHub Discussions**: Community announcement and discussion
- **Nushell Discord**: Nushell community notification
- **Social Media**: Twitter/X announcements for major releases

#### Timing
- **Release Day**: GitHub release and crates.io publication
- **Day 1**: Community announcements and social media
- **Week 1**: Follow-up on feedback and issues

---

This release procedure ensures consistent, high-quality releases while maintaining security, performance, and user satisfaction. Regular review and improvement of these procedures help maintain release excellence.