# Integration test script for nu_plugin_ulid (Windows PowerShell)
# Tests actual plugin installation and execution with Nushell

param(
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"

Write-Host "🧪 Running integration tests for nu_plugin_ulid" -ForegroundColor Cyan
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
$pluginPath = "$env:USERPROFILE\.cargo\bin\nu_plugin_ulid.exe"
Write-Host "🔗 Plugin path: $pluginPath" -ForegroundColor Yellow

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
if (-not (Test-Command "Test 1: Plugin registration" { 
    & nu -c "plugin add `"$pluginPath`""
})) { exit 1 }

# Test 2: Plugin info
if (-not (Test-Command "Test 2: Plugin info" { 
    & nu -c "plugin use ulid; ulid info"
})) { exit 1 }

# Test 3: ULID generation
$ulid = & nu -c "plugin use ulid; ulid generate" 2>$null
if ($ulid.Length -eq 26) {
    Write-Host "✅ Test 3: ULID generation successful: $ulid" -ForegroundColor Green
} else {
    Write-Host "❌ Test 3: ULID generation failed or invalid length" -ForegroundColor Red
    exit 1
}

# Test 4: ULID validation
$validationResult = & nu -c "plugin use ulid; ulid validate '$ulid'" 2>$null
if ($validationResult -eq "true") {
    Write-Host "✅ Test 4: ULID validation successful" -ForegroundColor Green
} else {
    Write-Host "❌ Test 4: ULID validation failed" -ForegroundColor Red
    exit 1
}

# Test 5: ULID parsing
if (-not (Test-Command "Test 5: ULID parsing" { 
    & nu -c "plugin use ulid; ulid parse '$ulid'"
})) { exit 1 }

# Test 6: Bulk generation
if (-not (Test-Command "Test 6: Bulk generation" { 
    & nu -c "plugin use ulid; ulid generate --count 3"
})) { exit 1 }

# Test 7: Stream processing
if (-not (Test-Command "Test 7: Stream processing" { 
    & nu -c "plugin use ulid; echo ['$ulid', 'invalid'] | ulid stream validate"
})) { exit 1 }

# Test 8: Security advice
if (-not (Test-Command "Test 8: Security advice" { 
    & nu -c "plugin use ulid; ulid security-advice"
})) { exit 1 }

# Test 9: Inspect command
if (-not (Test-Command "Test 9: ULID inspection" { 
    & nu -c "plugin use ulid; ulid inspect '$ulid'"
})) { exit 1 }

# Test 10: Sort command
$ulid2 = & nu -c "plugin use ulid; ulid generate" 2>$null
if (-not (Test-Command "Test 10: ULID sorting" { 
    & nu -c "plugin use ulid; echo ['$ulid', '$ulid2'] | ulid sort"
})) { exit 1 }

Write-Host ""
Write-Host "🎉 All integration tests passed!" -ForegroundColor Green
Write-Host "✅ Plugin is working correctly with Nushell" -ForegroundColor Green
Write-Host ""
Write-Host "Plugin installed at: $pluginPath"
Write-Host "You can now use:"
Write-Host "  nu -c `"plugin use ulid; ulid generate`""
Write-Host "  nu -c `"plugin use ulid; ulid info`""