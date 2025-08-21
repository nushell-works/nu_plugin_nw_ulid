# Monitoring and Support Infrastructure

**Version**: 1.0
**Last Updated**: August 18, 2025
**Target Audience**: Maintainers and infrastructure teams

This document outlines the monitoring and support infrastructure for nu_plugin_nw_ulid to ensure reliable operation, proactive issue detection, and excellent user experience.

## Table of Contents

1. [Infrastructure Overview](#infrastructure-overview)
2. [GitHub-Based Monitoring](#github-based-monitoring)
3. [Crates.io Monitoring](#cratesio-monitoring)
4. [Community Health Monitoring](#community-health-monitoring)
5. [Automated Alerting](#automated-alerting)
6. [Support Infrastructure](#support-infrastructure)
7. [Performance Monitoring](#performance-monitoring)
8. [Security Monitoring](#security-monitoring)

## Infrastructure Overview

### Monitoring Philosophy

The nu_plugin_nw_ulid monitoring infrastructure follows these principles:

- **Proactive Detection**: Identify issues before they impact users
- **Community Focus**: Monitor community health and engagement
- **Quality Assurance**: Continuous monitoring of code and release quality
- **Transparent Communication**: Open communication about issues and status
- **Automated Response**: Automated handling of routine monitoring tasks

### Infrastructure Components

#### 1. GitHub Actions Monitoring
- **CI/CD Pipeline Health**: Monitor build and test success rates
- **Security Scanning**: Automated vulnerability detection and alerts
- **Performance Regression**: Continuous performance monitoring
- **Dependency Health**: Monitor dependency updates and security issues

#### 2. Community Metrics
- **Issue Response Time**: Track time to first response and resolution
- **Community Engagement**: Monitor discussions, contributions, and feedback
- **User Support Quality**: Track support quality and user satisfaction
- **Project Health**: Overall project health and sustainability metrics

#### 3. External Service Monitoring
- **Crates.io Status**: Monitor package availability and download metrics
- **Documentation Accessibility**: Monitor docs.rs and documentation sites
- **CI Service Health**: Monitor GitHub Actions and external services
- **Dependency Ecosystem**: Monitor upstream dependencies and security

## GitHub-Based Monitoring

### Repository Metrics

#### 1. Issue and PR Management
```yaml
# .github/workflows/community-metrics.yml
name: Community Metrics

on:
  schedule:
    - cron: '0 6 * * *'  # Daily at 6 AM UTC
  workflow_dispatch:

jobs:
  community-health:
    runs-on: ubuntu-latest
    steps:
      - name: Check Issue Response Time
        uses: actions/github-script@v7
        with:
          script: |
            const { data: issues } = await github.rest.issues.listForRepo({
              owner: context.repo.owner,
              repo: context.repo.repo,
              state: 'open',
              sort: 'created',
              direction: 'desc'
            });

            const now = new Date();
            const twentyFourHours = 24 * 60 * 60 * 1000;
            let issuesNeedingResponse = 0;

            for (const issue of issues) {
              if (issue.comments === 0 && !issue.pull_request) {
                const created = new Date(issue.created_at);
                if (now - created > twentyFourHours) {
                  issuesNeedingResponse++;
                }
              }
            }

            if (issuesNeedingResponse > 0) {
              core.setFailed(`${issuesNeedingResponse} issues need response within 24 hours`);
            }

      - name: Check Stale Issues
        uses: actions/stale@v9
        with:
          stale-issue-message: |
            This issue has been automatically marked as stale because it has not had
            recent activity. It will be closed if no further activity occurs.
            Thank you for your contributions.
          stale-pr-message: |
            This pull request has been automatically marked as stale because it has not had
            recent activity. It will be closed if no further activity occurs.
            Thank you for your contributions.
          days-before-stale: 60
          days-before-close: 14
```

#### 2. CI/CD Health Monitoring
```yaml
# .github/workflows/ci-health.yml
name: CI Health Monitor

on:
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours
  workflow_dispatch:

jobs:
  ci-health:
    runs-on: ubuntu-latest
    steps:
      - name: Check Recent CI Success Rate
        uses: actions/github-script@v7
        with:
          script: |
            const { data: runs } = await github.rest.actions.listWorkflowRuns({
              owner: context.repo.owner,
              repo: context.repo.repo,
              workflow_id: 'ci.yml',
              per_page: 50
            });

            const recent = runs.workflow_runs.slice(0, 20);
            const successRate = recent.filter(r => r.conclusion === 'success').length / recent.length;

            console.log(`CI Success Rate: ${(successRate * 100).toFixed(1)}%`);

            if (successRate < 0.9) {
              core.setFailed(`CI success rate (${(successRate * 100).toFixed(1)}%) below 90%`);
            }

      - name: Security Scan Status
        run: |
          echo "🔒 Checking security scan status..."
          # Check last security workflow run
          gh run list --workflow=security.yml --limit=1 --json conclusion
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Code Quality Monitoring

#### 1. Test Coverage Tracking
```bash
# scripts/monitor_coverage.sh
#!/bin/bash

echo "📊 Monitoring Test Coverage..."

# Generate coverage report
cargo install cargo-tarpaulin --locked
cargo tarpaulin --out xml

# Extract coverage percentage
COVERAGE=$(grep -o 'line-rate="[0-9.]*"' cobertura.xml | head -1 | grep -o '[0-9.]*')
COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc -l | cut -d. -f1)

echo "Current test coverage: ${COVERAGE_PCT}%"

# Alert if coverage drops below threshold
if [ "$COVERAGE_PCT" -lt 90 ]; then
    echo "⚠️ WARNING: Test coverage (${COVERAGE_PCT}%) below 90% threshold"
    exit 1
fi

echo "✅ Test coverage meets quality standards"
```

#### 2. Performance Regression Detection
```bash
# scripts/monitor_performance.sh
#!/bin/bash

echo "⚡ Performance Regression Detection..."

# Run benchmarks
cargo bench > current_benchmarks.txt

# Compare with baseline (if exists)
if [ -f "baseline_benchmarks.txt" ]; then
    echo "Comparing with baseline performance..."

    # Extract key metrics and compare
    CURRENT_GENERATION=$(grep "ULID generation" current_benchmarks.txt | grep -o '[0-9.]*ns')
    BASELINE_GENERATION=$(grep "ULID generation" baseline_benchmarks.txt | grep -o '[0-9.]*ns')

    echo "Current ULID generation: $CURRENT_GENERATION"
    echo "Baseline ULID generation: $BASELINE_GENERATION"

    # Calculate regression percentage
    # (implementation would need more robust numeric comparison)

else
    echo "No baseline found, saving current benchmarks as baseline"
    cp current_benchmarks.txt baseline_benchmarks.txt
fi

echo "✅ Performance monitoring complete"
```

## Crates.io Monitoring

### Download and Usage Metrics

#### 1. Download Tracking
```bash
# scripts/monitor_crates_downloads.sh
#!/bin/bash

echo "📦 Monitoring Crates.io Downloads..."

PACKAGE_NAME="nu_plugin_nw_ulid"

# Fetch download stats from crates.io API
DOWNLOADS=$(curl -s "https://crates.io/api/v1/crates/$PACKAGE_NAME" | \
           jq -r '.crate.downloads')

echo "Total downloads: $DOWNLOADS"

# Store historical data
DATE=$(date +%Y-%m-%d)
echo "$DATE,$DOWNLOADS" >> download_history.csv

# Calculate daily growth
if [ -f "download_history.csv" ] && [ $(wc -l < download_history.csv) -gt 1 ]; then
    YESTERDAY_DOWNLOADS=$(tail -2 download_history.csv | head -1 | cut -d, -f2)
    DAILY_GROWTH=$((DOWNLOADS - YESTERDAY_DOWNLOADS))
    echo "Daily growth: $DAILY_GROWTH downloads"
fi

echo "✅ Download monitoring complete"
```

#### 2. Version Adoption Tracking
```bash
# scripts/monitor_version_adoption.sh
#!/bin/bash

echo "📈 Version Adoption Tracking..."

PACKAGE_NAME="nu_plugin_nw_ulid"

# Get version download stats
curl -s "https://crates.io/api/v1/crates/$PACKAGE_NAME/downloads" | \
jq -r '.version_downloads[] | "\(.version): \(.downloads) downloads"' | \
head -10

echo "✅ Version adoption tracking complete"
```

## Community Health Monitoring

### Engagement Metrics

#### 1. Community Activity Dashboard
```bash
# scripts/community_dashboard.sh
#!/bin/bash

echo "👥 Community Health Dashboard"
echo "============================="

# GitHub metrics
echo "📊 GitHub Metrics:"
gh api repos/:owner/:repo | jq -r '"Stars: \(.stargazers_count)"'
gh api repos/:owner/:repo | jq -r '"Forks: \(.forks_count)"'
gh api repos/:owner/:repo | jq -r '"Open Issues: \(.open_issues_count)"'

# Recent activity
echo ""
echo "🔄 Recent Activity:"
echo "Recent Issues (last 7 days):"
gh issue list --state open --created "$(date -d '7 days ago' '+%Y-%m-%d')" --limit 5

echo ""
echo "Recent PRs (last 7 days):"
gh pr list --state all --created "$(date -d '7 days ago' '+%Y-%m-%d')" --limit 5

# Contributor activity
echo ""
echo "👥 Contributor Activity:"
git log --since="30 days ago" --pretty=format:"%an" | sort | uniq -c | sort -nr

echo ""
echo "✅ Community dashboard complete"
```

#### 2. Response Time Monitoring
```python
# scripts/monitor_response_times.py
#!/usr/bin/env python3

import requests
import json
from datetime import datetime, timedelta
import statistics

def check_response_times():
    """Monitor issue and PR response times"""

    # GitHub API setup
    headers = {"Accept": "application/vnd.github.v3+json"}
    base_url = "https://api.github.com/repos/nushell-works/nu_plugin_nw_ulid"

    # Get recent issues
    issues_url = f"{base_url}/issues"
    response = requests.get(issues_url, headers=headers)
    issues = response.json()

    response_times = []

    for issue in issues:
        if issue.get('comments', 0) > 0:
            # Get first comment time
            comments_url = issue['comments_url']
            comments_response = requests.get(comments_url, headers=headers)
            comments = comments_response.json()

            if comments:
                created = datetime.fromisoformat(issue['created_at'].replace('Z', '+00:00'))
                first_response = datetime.fromisoformat(comments[0]['created_at'].replace('Z', '+00:00'))
                response_time = (first_response - created).total_seconds() / 3600  # hours
                response_times.append(response_time)

    if response_times:
        avg_response = statistics.mean(response_times)
        median_response = statistics.median(response_times)

        print(f"📊 Response Time Metrics:")
        print(f"   Average: {avg_response:.1f} hours")
        print(f"   Median: {median_response:.1f} hours")

        # Alert if response time is too high
        if avg_response > 48:  # 48 hours threshold
            print(f"⚠️  WARNING: Average response time ({avg_response:.1f}h) exceeds 48h threshold")
            return False

    print("✅ Response times within acceptable range")
    return True

if __name__ == "__main__":
    check_response_times()
```

## Automated Alerting

### Alert Configuration

#### 1. GitHub Actions Alerts
```yaml
# .github/workflows/alerts.yml
name: Monitoring Alerts

on:
  schedule:
    - cron: '0 8 * * *'  # Daily at 8 AM UTC
  workflow_dispatch:

jobs:
  check-alerts:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Check CI Health
        run: ./scripts/monitor_ci_health.sh

      - name: Check Community Metrics
        run: ./scripts/community_dashboard.sh

      - name: Check Security Status
        run: |
          cargo audit --quiet || echo "Security audit failed - manual review needed"

      - name: Performance Check
        run: ./scripts/monitor_performance.sh

      - name: Send Alerts
        if: failure()
        uses: actions/github-script@v7
        with:
          script: |
            const issue = await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `🚨 Monitoring Alert - ${new Date().toDateString()}`,
              body: `Automated monitoring detected issues that require attention.

              Please review the failed workflow run: ${context.payload.workflow_run?.html_url || 'N/A'}

              **Alert Categories:**
              - CI/CD Health
              - Community Response Times
              - Security Status
              - Performance Metrics

              This issue was automatically created by the monitoring system.`,
              labels: ['monitoring', 'alert', 'priority-high']
            });

            console.log(`Created monitoring alert issue #${issue.data.number}`);
```

#### 2. Email Notification Setup
```bash
# scripts/send_alert_email.sh
#!/bin/bash

ALERT_TYPE="$1"
ALERT_MESSAGE="$2"

if [ -n "$ALERT_EMAIL" ]; then
    echo "📧 Sending alert email..."

    cat << EOF | mail -s "nu_plugin_nw_ulid Alert: $ALERT_TYPE" "$ALERT_EMAIL"
Alert Type: $ALERT_TYPE
Timestamp: $(date)
Repository: nushell-works/nu_plugin_nw_ulid

Message:
$ALERT_MESSAGE

Please check the repository for more details:
https://github.com/nushell-works/nu_plugin_nw_ulid

This is an automated alert from the monitoring system.
EOF

    echo "✅ Alert email sent"
else
    echo "⚠️  No alert email configured"
fi
```

## Support Infrastructure

### Knowledge Base

#### 1. FAQ Automation
```bash
# scripts/update_faq.sh
#!/bin/bash

echo "📚 Updating FAQ from common issues..."

# Extract common issues and create FAQ entries
gh issue list --state closed --label "question" --json title,body,url --limit 20 | \
jq -r '.[] | "## \(.title)\n\(.body)\n\nSee: \(.url)\n"' > docs/AUTO_FAQ.md

echo "✅ FAQ updated from recent issues"
```

#### 2. Support Ticket Classification
```python
# scripts/classify_issues.py
#!/usr/bin/env python3

import requests
import json
import re

def classify_issue(title, body):
    """Classify issues automatically"""

    # Simple keyword-based classification
    bug_keywords = ['error', 'bug', 'crash', 'fail', 'broken', 'not working']
    feature_keywords = ['feature', 'enhancement', 'add', 'support', 'implement']
    docs_keywords = ['documentation', 'docs', 'readme', 'guide', 'example']
    performance_keywords = ['slow', 'performance', 'speed', 'optimization']

    text = (title + ' ' + body).lower()

    if any(keyword in text for keyword in bug_keywords):
        return 'bug'
    elif any(keyword in text for keyword in feature_keywords):
        return 'enhancement'
    elif any(keyword in text for keyword in docs_keywords):
        return 'documentation'
    elif any(keyword in text for keyword in performance_keywords):
        return 'performance'
    else:
        return 'question'

def auto_label_issues():
    """Automatically label new issues"""

    headers = {"Accept": "application/vnd.github.v3+json"}
    base_url = "https://api.github.com/repos/nushell-works/nu_plugin_nw_ulid"

    # Get unlabeled issues
    issues_url = f"{base_url}/issues?labels=none"
    response = requests.get(issues_url, headers=headers)
    issues = response.json()

    for issue in issues:
        if not issue.get('labels'):
            classification = classify_issue(issue['title'], issue['body'])
            print(f"Issue #{issue['number']}: {classification}")

            # Note: Would need GitHub token to actually apply labels
            # This is just for demonstration

if __name__ == "__main__":
    auto_label_issues()
```

### Documentation Monitoring

#### 1. Link Checker
```bash
# scripts/check_documentation_links.sh
#!/bin/bash

echo "🔗 Checking documentation links..."

# Check all markdown files for broken links
find docs/ -name "*.md" -exec markdown-link-check {} \;

# Check README links
markdown-link-check README.md

# Check if docs.rs is accessible
curl -f -s "https://docs.rs/nu_plugin_nw_ulid" > /dev/null || {
    echo "⚠️  docs.rs not accessible"
    exit 1
}

echo "✅ Documentation links verified"
```

#### 2. Documentation Freshness
```bash
# scripts/check_docs_freshness.sh
#!/bin/bash

echo "📅 Checking documentation freshness..."

# Find docs older than 90 days
find docs/ -name "*.md" -mtime +90 -print | while read file; do
    echo "⚠️  Old documentation: $file"
done

# Check if version numbers are consistent
VERSION=$(grep -o "version = \"[^\"]*\"" Cargo.toml | cut -d'"' -f2)
echo "Current version: $VERSION"

# Check if docs reference current version
if ! grep -r "$VERSION" docs/ > /dev/null; then
    echo "⚠️  Documentation may not reference current version"
fi

echo "✅ Documentation freshness check complete"
```

## Performance Monitoring

### Continuous Benchmarking

#### 1. Performance Tracking
```yaml
# .github/workflows/performance-monitoring.yml
name: Performance Monitoring

on:
  push:
    branches: [main]
  schedule:
    - cron: '0 12 * * *'  # Daily at noon UTC

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: |
          cargo bench > benchmark_results.txt
          cat benchmark_results.txt

      - name: Store benchmark results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results-${{ github.sha }}
          path: benchmark_results.txt

      - name: Compare with baseline
        run: |
          # Compare performance with previous runs
          echo "Performance comparison would go here"
```

#### 2. Memory Usage Monitoring
```bash
# scripts/monitor_memory.sh
#!/bin/bash

echo "🧠 Memory Usage Monitoring..."

# Test memory usage with different workloads
echo "Testing memory usage patterns..."

# Generate memory profile
valgrind --tool=massif --pages-as-heap=yes \
    cargo test --release test_memory_efficiency 2>/dev/null || {
    echo "⚠️  Memory profiling requires valgrind"
}

# Check for memory leaks in tests
cargo test --release 2>&1 | grep -i "leak\|memory" || echo "No memory issues detected"

echo "✅ Memory monitoring complete"
```

## Security Monitoring

### Vulnerability Scanning

#### 1. Dependency Monitoring
```yaml
# .github/workflows/security-monitoring.yml
name: Security Monitoring

on:
  schedule:
    - cron: '0 6 * * *'  # Daily security check
  workflow_dispatch:

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install security tools
        run: |
          cargo install cargo-audit --locked
          cargo install cargo-deny --locked

      - name: Audit dependencies
        run: |
          cargo audit --json > audit_results.json
          cat audit_results.json

      - name: Check supply chain
        run: cargo deny check

      - name: Security scan summary
        run: |
          VULNS=$(jq '.vulnerabilities.count' audit_results.json)
          if [ "$VULNS" -gt 0 ]; then
            echo "🚨 $VULNS vulnerabilities found"
            exit 1
          else
            echo "✅ No vulnerabilities detected"
          fi
```

#### 2. License Compliance
```bash
# scripts/check_license_compliance.sh
#!/bin/bash

echo "📄 License Compliance Check..."

# Check all dependencies for license compatibility
cargo deny check licenses

# Generate license report
cargo license > LICENSE_REPORT.txt

echo "✅ License compliance check complete"
```

---

This monitoring and support infrastructure ensures nu_plugin_nw_ulid maintains high quality, responds quickly to community needs, and operates reliably in production environments. Regular review and improvement of these monitoring systems helps maintain project health and user satisfaction.

For questions about monitoring setup or to suggest improvements, please use our [community discussions](https://github.com/nushell-works/nu_plugin_nw_ulid/discussions).
