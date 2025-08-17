# Pull Request

## Description

Brief description of the changes in this PR.

## Type of Change

Please select the relevant option:

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 Documentation update (improvements or corrections to documentation)
- [ ] 📝 Script template (new template or example for common use cases)
- [ ] 🔧 Maintenance (dependency updates, CI improvements, etc.)
- [ ] 🎨 Code style/formatting changes
- [ ] ♻️ Refactoring (no functional changes)
- [ ] 🏗️ Infrastructure (CI/CD, deployment, build improvements)

## Related Issues

Closes #(issue number)
Relates to #(issue number)

## Changes Made

Detailed list of changes:

- Change 1
- Change 2
- Change 3

## Testing

### Test Coverage

- [ ] New tests added for new functionality
- [ ] Existing tests updated as needed
- [ ] All tests pass locally
- [ ] Manual testing completed

### Testing Instructions

Describe how reviewers can test these changes:

1. Step 1
2. Step 2
3. Step 3

### Test Results

```bash
# Paste relevant test output here
cargo test
cargo clippy
cargo audit
```

## Performance Impact

- [ ] No performance impact
- [ ] Performance improvement (provide benchmarks)
- [ ] Performance regression (justify why)
- [ ] Performance impact unknown/needs measurement

## Security Considerations

- [ ] No security implications
- [ ] Security improvement
- [ ] Potential security concern (describe below)

If there are security considerations, please describe:

## Breaking Changes

If this is a breaking change, please describe:

1. What breaks
2. How users should migrate
3. Why the breaking change is necessary

## Documentation

- [ ] Code comments updated
- [ ] README.md updated
- [ ] CHANGELOG.md updated
- [ ] API documentation updated
- [ ] Examples updated

## Checklist

### Code Quality

- [ ] Code follows the project's style guidelines
- [ ] Self-review of code completed
- [ ] Code is properly commented, particularly in hard-to-understand areas
- [ ] No new warnings introduced

### Testing & Validation

- [ ] All tests pass (`cargo test`)
- [ ] No linting errors (`cargo clippy`)
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] Security audit passes (`cargo audit`)
- [ ] Supply chain checks pass (`cargo deny check`)

### Documentation & Communication

- [ ] Changes are documented
- [ ] Commit messages follow conventional format
- [ ] PR title is descriptive
- [ ] Any necessary migration guides created

### Dependencies & Compatibility

- [ ] No unnecessary dependencies added
- [ ] Minimum supported Rust version maintained
- [ ] Nushell compatibility maintained
- [ ] Cross-platform compatibility verified

## Additional Notes

Any additional information, concerns, or questions for reviewers:

---

**For Maintainers:**

- [ ] Labels applied
- [ ] Milestone assigned (if applicable)
- [ ] Security review completed (if needed)
- [ ] Performance review completed (if needed)
- [ ] Breaking change process followed (if applicable)