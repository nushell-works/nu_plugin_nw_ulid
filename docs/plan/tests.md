# Comprehensive Test Plan for nu_plugin_ulid

## Overview

This document outlines a comprehensive testing strategy for the nu_plugin_ulid project, defining what should be tested as unit tests vs integration tests, and establishing clear coverage goals.

## Current State Analysis

### Existing Tests ✅
- **Core Engine Logic** (`src/ulid_engine.rs`) - 14 unit tests ✅
- **Security Functions** (`src/security.rs`) - 3 unit tests ✅  
- **Error Handling** (`src/error.rs`) - 2 unit tests ✅
- **Plugin Metadata** (`src/lib.rs`) - 2 unit tests ✅
- **Performance Tests** (`tests/performance_tests.rs`) - 9 integration tests ✅
- **Security Tests** (`tests/security_tests.rs`) - 8 integration tests ✅

### Missing Tests ❌
- **All Command Modules** (`src/commands/*.rs`) - 0 tests ❌
- **Main Entry Point** (`src/main.rs`) - 0 tests ❌

## Test Strategy Framework

### Unit Tests (`src/` with `#[cfg(test)]`)
**Purpose**: Test internal logic, algorithms, and data structures in isolation
**Location**: Inline with source code using `#[cfg(test)]` modules
**Access**: Can test private functions and internal state

### Integration Tests (`tests/` directory)
**Purpose**: Test plugin commands as end-users would interact with them
**Location**: Separate files in `tests/` directory
**Access**: Only public APIs, treating the plugin as external dependency

## Detailed Test Plan

### 1. Core Engine (Unit Tests) - EXISTING ✅

**File**: `src/ulid_engine.rs` 
**Status**: Well tested (14 tests)
**Coverage**: ~95% estimated

Tests cover:
- ULID generation (single, bulk, with timestamp)
- ULID validation (basic and detailed) 
- ULID parsing and component extraction
- Output format conversion
- Edge cases and error conditions

### 2. Security Module (Unit Tests) - EXISTING ✅

**File**: `src/security.rs`
**Status**: Basic tests (3 tests) 
**Coverage**: ~70% estimated

**Additional tests needed**:
- More security context keywords
- Edge cases in warning generation
- Security rating calculations

### 3. Error Handling (Unit Tests) - EXISTING ✅

**File**: `src/error.rs`
**Status**: Basic tests (2 tests)
**Coverage**: ~60% estimated

**Additional tests needed**:
- All error variant creation
- Error message formatting
- Error conversion chains

### 4. Command Modules (Unit Tests) - PARTIALLY COMPLETE ✅

#### 4.1 ULID Commands (`src/commands/ulid.rs`) - COMPLETED ✅
**Lines of Code**: 702 (largest command file)
**Commands**: 4 commands (Generate, Validate, Parse, SecurityAdvice)
**Tests Implemented**: 30 unit tests

**Unit Tests Completed**:
- ✅ Command signature validation
- ✅ Input parameter parsing
- ✅ Format string validation  
- ✅ Security context detection
- ✅ Error condition handling
- ✅ Output format generation
- ✅ Count validation logic
- ✅ Timestamp parameter validation
- ✅ ULID string pattern validation
- ✅ Parsing logic integration testing

**Approach**: Test command logic without Nushell runtime
**Priority**: HIGH (most complex commands) - DONE ✅

#### 4.2 Hash Commands (`src/commands/hash.rs`) - COMPLETED ✅
**Lines of Code**: 365
**Commands**: 4 commands (SHA256, SHA512, Blake3, Random)
**Tests Implemented**: 24 unit tests

**Unit Tests Completed**:
- ✅ Hash algorithm selection and command signatures
- ✅ Input validation (length limits)
- ✅ Binary vs string output flags
- ✅ Random byte generation and validation
- ✅ Error conditions and boundary testing
- ✅ Known test vector validation (SHA-256, BLAKE3)
- ✅ Deterministic hash computation testing
- ✅ Hex encoding correctness
- ✅ Cryptographic algorithm correctness

**Priority**: HIGH (crypto operations) - DONE ✅

#### 4.3 Time Commands (`src/commands/time.rs`) 
**Lines of Code**: 321
**Commands**: 3 commands (Now, Parse, Millis)

**Unit Tests Needed**:
- Timestamp parsing from various formats
- Timezone handling
- Output format conversion
- Edge cases (invalid dates, future dates)

**Priority**: MEDIUM

#### 4.4 Encoding Commands (`src/commands/encode.rs`)
**Lines of Code**: 308  
**Commands**: 4 commands (Base32 encode/decode, Hex encode/decode)

**Unit Tests Needed**:
- Encoding/decoding correctness
- Invalid input handling
- Case sensitivity options
- Empty input handling

**Priority**: MEDIUM

#### 4.5 Other Commands (Lower Priority)
- **UUID Commands** (`src/commands/uuid.rs`) - 179 lines, 3 commands
- **Sort Commands** (`src/commands/sort.rs`) - 476 lines, 2 commands  
- **Stream Commands** (`src/commands/stream.rs`) - 503 lines, 2 commands
- **Info Commands** (`src/commands/info.rs`) - 69 lines, 1 command

### 5. Integration Tests (Command Execution) - PARTIAL ✅

#### 5.1 Existing Integration Tests ✅
- **Performance Tests**: Command execution speed benchmarks
- **Security Tests**: Security boundary testing

#### 5.2 Missing Integration Tests ❌

**File**: `tests/command_integration_tests.rs` 
**Purpose**: Test actual command execution through Nushell plugin interface

**Test Categories**:

##### Core ULID Workflow Tests
```bash
# Test the complete user workflow
ulid generate
ulid generate --count 5
ulid validate "01AN4Z07BY79KA1307SR9X4MV3"
ulid parse "01AN4Z07BY79KA1307SR9X4MV3"
```

##### Cross-Command Integration
```bash
# Test commands working together
ulid generate | ulid validate
ulid generate --format json | get timestamp | ulid time parse
```

##### Error Handling Integration
```bash
# Test error paths through real plugin interface
ulid validate "invalid"
ulid parse "invalid"  
ulid generate --count -1
```

**File**: `tests/plugin_lifecycle_tests.rs`
**Purpose**: Test plugin loading, command registration, cleanup

## Implementation Priority

### Phase 1: Critical Command Unit Tests
1. ✅ **ULID Commands** (`src/commands/ulid.rs`) - Core functionality - DONE (30 tests)
2. ✅ **Hash Commands** (`src/commands/hash.rs`) - Security-critical - DONE (24 tests)
3. **Time Commands** (`src/commands/time.rs`) - Core functionality - IN PROGRESS

### Phase 2: Supporting Command Unit Tests  
4. **Encoding Commands** (`src/commands/encode.rs`)
5. **UUID Commands** (`src/commands/uuid.rs`)
6. **Sort Commands** (`src/commands/sort.rs`)

### Phase 3: Integration and Edge Cases
7. **Command Integration Tests** (`tests/command_integration_tests.rs`)
8. **Plugin Lifecycle Tests** (`tests/plugin_lifecycle_tests.rs`)
9. **Stream Commands** (`src/commands/stream.rs`) 
10. **Info Commands** (`src/commands/info.rs`)

### Phase 4: Enhanced Coverage
11. Enhanced security module tests
12. Enhanced error handling tests
13. Cross-platform compatibility tests

## Coverage Goals

### Current Estimated Coverage: ~65%
- Core engine: ~95% ✅
- Security: ~80% ✅ (improved with context detection tests)
- Error handling: ~70% ✅ (improved with command error tests) 
- Commands: ~35% ✅ (54 tests added for ULID + Hash commands)
- Integration: ~20% ❌

### Target Coverage: ~85%
- Core engine: ~95% (maintain)
- Security: ~90% (improve)
- Error handling: ~85% (improve) 
- Commands: ~80% (new)
- Integration: ~70% (improve)

## Test Implementation Guidelines

### Unit Test Standards
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]  
    fn test_specific_function() {
        // Arrange
        let input = "test_input";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
        assert!(condition);
    }
}
```

### Integration Test Standards
```rust
use nu_test_support::nu_with_plugins;

#[test]
fn test_ulid_generate_command() {
    let actual = nu_with_plugins(
        cwd: "tests/fixtures", 
        plugins: vec![UlidPlugin::new()],
        "ulid generate"
    );
    
    assert!(actual.out.len() == 26);
    assert!(!actual.err.contains("error"));
}
```

### Test Data Management
- **Fixtures**: Store test ULIDs, timestamps, and expected outputs in `tests/fixtures/`
- **Mocking**: Mock external dependencies (random number generation, system time)
- **Deterministic**: Use fixed seeds/timestamps where possible for reproducible tests

## Test Automation

### CI Integration
- Run all tests on every PR
- Enforce minimum coverage thresholds
- Test on multiple platforms (Linux, macOS, Windows)
- Test with multiple Nushell versions

### Coverage Reporting
- Use `tarpaulin` or `llvm-cov` for coverage collection
- Upload to Codecov for tracking
- Fail CI if coverage drops below thresholds

### Performance Regression Detection
- Benchmark key operations in CI
- Alert on performance regressions >10%
- Track performance trends over time

## Success Metrics

### Quantitative Metrics
- **Line Coverage**: >80% overall
- **Branch Coverage**: >75% for critical paths  
- **Function Coverage**: >90% for public APIs
- **Integration Coverage**: >70% of command combinations

### Qualitative Metrics
- All error paths tested
- All security boundaries validated  
- All user-facing commands have integration tests
- Documentation examples have corresponding tests

## Risk Assessment

### High Risk Areas (Need Thorough Testing)
1. **Cryptographic Operations** - Hash generation, randomness
2. **Input Validation** - ULID parsing, format validation
3. **Security Context Detection** - Sensitive operation warnings
4. **Cross-Platform Time Handling** - Timezone, epoch conversions

### Medium Risk Areas
1. **Output Format Conversion** - JSON, binary, string formats
2. **Bulk Operations** - Large dataset handling
3. **Error Propagation** - Consistent error handling

### Low Risk Areas
1. **Plugin Metadata** - Version, description strings
2. **Help Text Generation** - Command documentation
3. **Simple Utility Functions** - String manipulation

## Progress Summary

### ✅ COMPLETED (Phase 1 - Critical Commands)
- **ULID Commands**: 30 unit tests covering all 4 commands (Generate, Validate, Parse, SecurityAdvice)
- **Hash Commands**: 24 unit tests covering all 4 commands (SHA256, SHA512, Blake3, Random)
- **Total Tests Added**: 54 new unit tests
- **Coverage Improvement**: ~40% → ~65% (estimated 25% improvement)

### 🚧 IN PROGRESS  
- **Time Commands**: Next priority (3 commands)

### ⏳ REMAINING WORK
- **Encoding Commands**: 4 commands (Base32/Hex encode/decode)
- **UUID Commands**: 3 commands  
- **Sort Commands**: 2 commands
- **Stream Commands**: 2 commands
- **Info Commands**: 1 command
- **Integration Tests**: Command workflow testing

### 🎯 Impact
The critical Phase 1 implementation has:
1. **Eliminated the biggest coverage gap** - Command modules went from 0% to ~35% coverage
2. **Added comprehensive testing** for the most complex user-facing functionality
3. **Improved overall project coverage** by an estimated 25 percentage points
4. **Established testing patterns** for remaining command modules

## Conclusion

This comprehensive test plan will improve the project's test coverage from ~40% to ~85%, focusing first on the most critical and untested components (command modules), then expanding to full integration testing. The phased approach ensures immediate improvement in the most important areas while building toward comprehensive coverage.

**Phase 1 is now 67% complete** with the two highest-priority command modules fully tested.