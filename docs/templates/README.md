# ULID Script Templates and Integration Examples

This directory contains ready-to-use script templates and integration examples for common ULID use cases. These templates demonstrate best practices and can be customized for your specific needs.

## Template Categories

### 📊 Data Management
- [Database Migration](#database-migration) - Migrate legacy systems to ULID-based primary keys
- [Data Validation](#data-validation) - Validate and clean ULID data in datasets
- [Batch Processing](#batch-processing) - Process large datasets efficiently
- [ETL Pipeline](#etl-pipeline) - Extract, transform, and load data with ULID tracking

### 🔧 API & Web Services
- [REST API Integration](#rest-api-integration) - Integrate ULIDs in REST API workflows
- [Request Tracking](#request-tracking) - Track requests across microservices
- [Rate Limiting](#rate-limiting) - Implement rate limiting with ULID timestamps
- [API Gateway](#api-gateway) - Route and track API requests

### 🖥️ System Administration
- [Log Analysis](#log-analysis) - Analyze logs with ULID request IDs
- [Health Monitoring](#health-monitoring) - Monitor system health with ULID tracking
- [Backup Management](#backup-management) - Manage backups with ULID versioning
- [Performance Monitoring](#performance-monitoring) - Track performance metrics

### 🔐 Security & Audit
- [Audit Trail](#audit-trail) - Implement comprehensive audit logging
- [Security Scanning](#security-scanning) - Scan for security issues in ULID usage
- [Access Control](#access-control) - Manage access with ULID-based tokens
- [Compliance Reporting](#compliance-reporting) - Generate compliance reports

### 🚀 DevOps & CI/CD
- [Build Tracking](#build-tracking) - Track builds and deployments
- [Container Management](#container-management) - Manage containers with ULID tags
- [Release Management](#release-management) - Manage software releases
- [Infrastructure Monitoring](#infrastructure-monitoring) - Monitor infrastructure

## Quick Start

1. **Choose a template** from the categories above
2. **Copy the template** to your project directory
3. **Customize** the configuration and parameters
4. **Test** with your data
5. **Deploy** to production

## Template Structure

Each template follows this structure:

```
template-name.nu
├── Configuration section    # Customizable parameters
├── Helper functions        # Reusable utility functions
├── Main logic             # Core functionality
├── Error handling         # Comprehensive error management
├── Testing section        # Built-in tests and validation
└── Usage examples         # Practical usage examples
```

## Installation

1. **Install the ULID plugin:**
   ```bash
   cargo install nu_plugin_nw_ulid
   plugin add ~/.cargo/bin/nu_plugin_nw_ulid
   plugin use nw_ulid
   ```

2. **Download templates:**
   ```bash
   git clone https://github.com/nushell-works/nu_plugin_nw_ulid.git
   cd nu_plugin_nw_ulid/docs/templates
   ```

3. **Copy desired template:**
   ```bash
   cp api-request-tracking.nu ~/my-project/
   ```

## Template List

| Template | Category | Use Case | Complexity |
|----------|----------|----------|------------|
| [database-migration.nu](database-migration.nu) | Data Management | Migrate to ULID primary keys | Medium |
| [api-request-tracking.nu](api-request-tracking.nu) | API & Web | Track API requests | Easy |
| [log-analysis.nu](log-analysis.nu) | System Admin | Analyze application logs | Medium |
| [audit-trail.nu](audit-trail.nu) | Security | Comprehensive audit logging | Hard |
| [build-tracking.nu](build-tracking.nu) | DevOps | Track CI/CD builds | Medium |
| [data-validation.nu](data-validation.nu) | Data Management | Validate ULID datasets | Easy |
| [performance-monitoring.nu](performance-monitoring.nu) | System Admin | Monitor performance metrics | Medium |
| [etl-pipeline.nu](etl-pipeline.nu) | Data Management | ETL with ULID tracking | Hard |
| [microservice-orchestration.nu](microservice-orchestration.nu) | API & Web | Orchestrate microservices | Hard |
| [backup-management.nu](backup-management.nu) | System Admin | Manage backups with ULIDs | Medium |

## Best Practices

### 1. Configuration Management
```nu
# Always use a configuration section at the top
const CONFIG = {
    batch_size: 1000,
    max_retries: 3,
    timeout_seconds: 30,
    log_level: "info"
}
```

### 2. Error Handling
```nu
# Implement comprehensive error handling
def safe_operation [data: any] {
    try {
        # Your operation here
        $data | process_data
    } catch { |e|
        log error $"Operation failed: ($e.msg)"
        { success: false, error: $e.msg, data: null }
    }
}
```

### 3. Logging and Monitoring
```nu
# Include logging for observability
def log [level: string, message: string] {
    let timestamp = date now | format date "%Y-%m-%d %H:%M:%S"
    let log_entry = $"[$timestamp] [($level | str upcase)] ($message)"

    print $log_entry
    $log_entry | save --append application.log
}
```

### 4. Testing
```nu
# Include test functions
def test_ulid_generation [] {
    let ulid = ulid generate
    assert (ulid validate $ulid) "Generated ULID should be valid"
    print "✅ ULID generation test passed"
}
```

### 5. Performance Optimization
```nu
# Use streaming for large datasets
def process_large_dataset [data: list] {
    $data
    | chunks $CONFIG.batch_size
    | each { |chunk|
        $chunk | ulid stream validate --parallel
    }
    | flatten
}
```

## Contributing Templates

We welcome contributions of new templates! To contribute:

1. **Fork the repository**
2. **Create your template** following the structure above
3. **Test thoroughly** with various datasets
4. **Document usage** with clear examples
5. **Submit a pull request**

### Template Guidelines

- **Clear documentation** with usage examples
- **Error handling** for all edge cases
- **Performance considerations** for large datasets
- **Security best practices** where applicable
- **Test coverage** with example data
- **Modular design** with reusable functions

## Support

- 📖 [User Guide](../USER_GUIDE.md) - Comprehensive user documentation
- 🔧 [API Reference](../scripting/api.md) - Complete API documentation
- 💬 [Discussions](https://github.com/nushell-works/nu_plugin_nw_ulid/discussions) - Community support
- 🐛 [Issues](https://github.com/nushell-works/nu_plugin_nw_ulid/issues) - Bug reports and feature requests

---

**Note**: All templates are provided as examples and should be customized for your specific use case. Always test thoroughly in a development environment before using in production.
