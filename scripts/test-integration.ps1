# Integration test script for nu_plugin_nw_ulid (Windows PowerShell)
# Tests actual plugin installation and execution with Nushell

param(
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"

Write-Host "🧪 Running integration tests for nu_plugin_nw_ulid" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan

# Check if Nushell is installed
try {
    $nuVersion = & nu --version 2>$null | Select-Object -First 1
    Write-Host "📋 Found: $nuVersion" -ForegroundColor Yellow
} catch {
    Write-Host "❌ Nushell not found. Please install Nushell 0.106.1+" -ForegroundColor Red
    Write-Host "Install with: cargo install nu --version 0.106.1" -ForegroundColor Yellow
    exit 1
}

# Build the plugin
Write-Host "🔨 Building plugin..." -ForegroundColor Yellow
& cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Plugin build failed" -ForegroundColor Red
    exit 1
}

# Install the plugin
Write-Host "📦 Installing plugin..." -ForegroundColor Yellow
& cargo install --path .
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Plugin installation failed" -ForegroundColor Red
    exit 1
}

# Get the plugin path
$pluginPath = "$env:USERPROFILE\.cargo\bin\nu_plugin_nw_ulid.exe"
Write-Host "🔗 Plugin path: $pluginPath" -ForegroundColor Yellow

# Ensure Nushell config directory exists
Write-Host "📁 Setting up Nushell configuration..." -ForegroundColor Yellow
$nuConfigDir = "$env:APPDATA\nushell"
Write-Host "Config directory: $nuConfigDir" -ForegroundColor Cyan
if (-not (Test-Path $nuConfigDir)) {
    Write-Host "Creating config directory..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $nuConfigDir -Force | Out-Null
} else {
    Write-Host "Config directory already exists" -ForegroundColor Green
}

# Initialize Nushell if needed (this creates the plugin registry)
$pluginRegistry = "$nuConfigDir\plugin.msgpackz"
Write-Host "Plugin registry path: $pluginRegistry" -ForegroundColor Cyan
if (-not (Test-Path $pluginRegistry)) {
    Write-Host "🔧 Initializing Nushell configuration..." -ForegroundColor Yellow
    try {
        $initResult = & nu -c "version" 2>&1
        Write-Host "Nushell initialization result: $initResult" -ForegroundColor Cyan
    } catch {
        Write-Host "Error during Nushell initialization: $_" -ForegroundColor Red
    }
    # Check if plugin registry was created
    if (Test-Path $pluginRegistry) {
        Write-Host "Plugin registry created successfully" -ForegroundColor Green
    } else {
        Write-Host "Plugin registry was not created" -ForegroundColor Yellow
        # Don't create an invalid empty file - let Nushell create it properly
    }
} else {
    Write-Host "Plugin registry already exists" -ForegroundColor Green
}

# Test function
function Test-Command {
    param(
        [string]$TestName,
        [scriptblock]$Command,
        [string]$ExpectedPattern = $null
    )

    Write-Host "🧪 $TestName" -ForegroundColor Yellow

    try {
        $result = & $Command 2>$null
        if ($LASTEXITCODE -eq 0) {
            if ($ExpectedPattern -and $result -notmatch $ExpectedPattern) {
                Write-Host "❌ $TestName failed - unexpected output" -ForegroundColor Red
                return $false
            }
            Write-Host "✅ $TestName successful" -ForegroundColor Green
            return $true
        } else {
            Write-Host "❌ $TestName failed" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "❌ $TestName failed with exception: $_" -ForegroundColor Red
        return $false
    }
}

# Test 1: Plugin registration
Write-Host "🧪 Test 1: Plugin registration" -ForegroundColor Yellow
try {
    # Use single quotes to avoid backslash escape issues
    $result = & nu -c "plugin add '$pluginPath'" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Test 1: Plugin registration successful" -ForegroundColor Green
    } else {
        Write-Host "❌ Test 1: Plugin registration failed" -ForegroundColor Red
        Write-Host "Error output: $result" -ForegroundColor Red

        # If the plugin registry is corrupted, try removing it and retry
        if ($result -match "Error while reading plugin registry file") {
            Write-Host "🔧 Plugin registry appears corrupted, removing and retrying..." -ForegroundColor Yellow
            if (Test-Path $pluginRegistry) {
                Remove-Item $pluginRegistry -Force
            }
            # Retry plugin registration
            $retryResult = & nu -c "plugin add '$pluginPath'" 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Host "✅ Test 1: Plugin registration successful on retry" -ForegroundColor Green
            } else {
                Write-Host "❌ Test 1: Plugin registration failed on retry" -ForegroundColor Red
                Write-Host "Retry error output: $retryResult" -ForegroundColor Red
                exit 1
            }
        } else {
            exit 1
        }
    }
} catch {
    Write-Host "❌ Test 1: Plugin registration failed with exception: $_" -ForegroundColor Red
    Write-Host "Error output: $result" -ForegroundColor Red
    exit 1
}

# Test 2: Plugin info
if (-not (Test-Command "Test 2: Plugin info" {
    & nu -c "plugin use nw_ulid; ulid info"
})) { exit 1 }

# Test 3: ULID generation
$ulid = & nu -c "plugin use nw_ulid; ulid generate" 2>$null
if ($ulid.Length -eq 26) {
    Write-Host "✅ Test 3: ULID generation successful: $ulid" -ForegroundColor Green
} else {
    Write-Host "❌ Test 3: ULID generation failed or invalid length" -ForegroundColor Red
    exit 1
}

# Test 4: ULID validation
$validationResult = & nu -c "plugin use nw_ulid; ulid validate '$ulid'" 2>$null
if ($validationResult -eq "true") {
    Write-Host "✅ Test 4: ULID validation successful" -ForegroundColor Green
} else {
    Write-Host "❌ Test 4: ULID validation failed" -ForegroundColor Red
    exit 1
}

# Test 5: ULID parsing
if (-not (Test-Command "Test 5: ULID parsing" {
    & nu -c "plugin use nw_ulid; ulid parse '$ulid'"
})) { exit 1 }

# Test 6: Bulk generation
if (-not (Test-Command "Test 6: Bulk generation" {
    & nu -c "plugin use nw_ulid; ulid generate --count 3"
})) { exit 1 }

# Test 7: Stream processing
if (-not (Test-Command "Test 7: Stream processing" {
    & nu -c "plugin use nw_ulid; echo ['$ulid', 'invalid'] | ulid stream validate"
})) { exit 1 }

# Test 8: Security advice
if (-not (Test-Command "Test 8: Security advice" {
    & nu -c "plugin use nw_ulid; ulid security-advice"
})) { exit 1 }

# Test 9: Inspect command
if (-not (Test-Command "Test 9: ULID inspection" {
    & nu -c "plugin use nw_ulid; ulid inspect '$ulid'"
})) { exit 1 }

# Test 10: Sort command
$ulid2 = & nu -c "plugin use nw_ulid; ulid generate" 2>$null
if (-not (Test-Command "Test 10: ULID sorting" {
    & nu -c "plugin use nw_ulid; echo ['$ulid', '$ulid2'] | ulid sort"
})) { exit 1 }

Write-Host ""
Write-Host "🎉 All integration tests passed!" -ForegroundColor Green
Write-Host "✅ Plugin is working correctly with Nushell" -ForegroundColor Green
Write-Host ""
Write-Host "Plugin installed at: $pluginPath"
Write-Host "You can now use:"
Write-Host "  nu -c `"plugin use nw_ulid; ulid generate`""
Write-Host "  nu -c `"plugin use nw_ulid; ulid info`""
