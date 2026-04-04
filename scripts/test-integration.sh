#!/bin/bash
set -euo pipefail

# Integration test script for nu_plugin_nw_ulid
# Tests actual plugin installation and execution with Nushell

echo "🧪 Running integration tests for nu_plugin_nw_ulid"
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Nushell is installed
if ! command -v nu &> /dev/null; then
    echo -e "${RED}❌ Nushell not found. Please install Nushell 0.110.0+${NC}"
    echo "Install with: cargo install nu --version 0.110.0"
    exit 1
fi

# Check Nushell version
NU_VERSION=$(nu --version | head -1)
echo -e "${YELLOW}📋 Found: $NU_VERSION${NC}"

# Build the plugin
echo -e "${YELLOW}🔨 Building plugin...${NC}"
cargo build --release --locked

# Install the plugin
echo -e "${YELLOW}📦 Installing plugin...${NC}"
cargo install --path . --locked

# Get the plugin path
PLUGIN_PATH="$HOME/.cargo/bin/nu_plugin_nw_ulid"
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    PLUGIN_PATH="$HOME/.cargo/bin/nu_plugin_nw_ulid.exe"
fi

echo -e "${YELLOW}🔗 Plugin path: $PLUGIN_PATH${NC}"

# Ensure Nushell config directory exists
echo -e "${YELLOW}📁 Setting up Nushell configuration...${NC}"
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS uses Application Support directory
    NU_CONFIG_DIR="$HOME/Library/Application Support/nushell"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    # Windows (if running under MSYS/Cygwin)
    NU_CONFIG_DIR="$APPDATA/nushell"
else
    # Linux and other Unix-like systems
    NU_CONFIG_DIR="$HOME/.config/nushell"
fi

mkdir -p "$NU_CONFIG_DIR"

# Initialize Nushell if needed (this creates the plugin registry)
if [[ ! -f "$NU_CONFIG_DIR/plugin.msgpackz" ]]; then
    echo -e "${YELLOW}🔧 Initializing Nushell configuration...${NC}"
    nu -c "version" > /dev/null 2>&1 || true
fi

# Test 1: Plugin registration
echo -e "${YELLOW}🧪 Test 1: Plugin registration${NC}"
if nu -c "plugin add $PLUGIN_PATH"; then
    echo -e "${GREEN}✅ Plugin registration successful${NC}"
else
    echo -e "${RED}❌ Plugin registration failed${NC}"
    exit 1
fi

# Test 2: Plugin info
echo -e "${YELLOW}🧪 Test 2: Plugin info${NC}"
if nu -c "plugin use nw_ulid; ulid info" &> /dev/null; then
    echo -e "${GREEN}✅ Plugin info command works${NC}"
else
    echo -e "${RED}❌ Plugin info command failed${NC}"
    exit 1
fi

# Test 3: ULID generation
echo -e "${YELLOW}🧪 Test 3: ULID generation${NC}"
ULID=$(nu -c "plugin use nw_ulid; ulid generate" 2>/dev/null)
if [[ ${#ULID} -eq 26 ]]; then
    echo -e "${GREEN}✅ ULID generation successful: $ULID${NC}"
else
    echo -e "${RED}❌ ULID generation failed or invalid length${NC}"
    exit 1
fi

# Test 4: ULID validation
echo -e "${YELLOW}🧪 Test 4: ULID validation${NC}"
if nu -c "plugin use nw_ulid; ulid validate '$ULID'" | grep -q "true"; then
    echo -e "${GREEN}✅ ULID validation successful${NC}"
else
    echo -e "${RED}❌ ULID validation failed${NC}"
    exit 1
fi

# Test 5: ULID parsing
echo -e "${YELLOW}🧪 Test 5: ULID parsing${NC}"
if nu -c "plugin use nw_ulid; ulid parse '$ULID'" &> /dev/null; then
    echo -e "${GREEN}✅ ULID parsing successful${NC}"
else
    echo -e "${RED}❌ ULID parsing failed${NC}"
    exit 1
fi

# Test 6: Bulk generation
echo -e "${YELLOW}🧪 Test 6: Bulk generation${NC}"
if nu -c "plugin use nw_ulid; ulid generate --count 3" &> /dev/null; then
    echo -e "${GREEN}✅ Bulk generation successful${NC}"
else
    echo -e "${RED}❌ Bulk generation failed${NC}"
    exit 1
fi

# Test 7: Security advice
echo -e "${YELLOW}🧪 Test 7: Security advice${NC}"
if nu -c "plugin use nw_ulid; ulid security-advice" &> /dev/null; then
    echo -e "${GREEN}✅ Security advice command works${NC}"
else
    echo -e "${RED}❌ Security advice command failed${NC}"
    exit 1
fi

# Test 8: Inspect command
echo -e "${YELLOW}🧪 Test 8: ULID inspection${NC}"
if nu -c "plugin use nw_ulid; ulid inspect '$ULID'" &> /dev/null; then
    echo -e "${GREEN}✅ ULID inspection successful${NC}"
else
    echo -e "${RED}❌ ULID inspection failed${NC}"
    exit 1
fi

# Test 9: Sort command
echo -e "${YELLOW}🧪 Test 9: ULID sorting${NC}"
ULID2=$(nu -c "plugin use nw_ulid; ulid generate" 2>/dev/null)
if nu -c "plugin use nw_ulid; echo ['$ULID', '$ULID2'] | ulid sort" &> /dev/null; then
    echo -e "${GREEN}✅ ULID sorting successful${NC}"
else
    echo -e "${RED}❌ ULID sorting failed${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 All integration tests passed!${NC}"
echo -e "${GREEN}✅ Plugin is working correctly with Nushell${NC}"
echo ""
echo "Plugin installed at: $PLUGIN_PATH"
echo "You can now use:"
echo "  nu -c \"plugin use nw_ulid; ulid generate\""
echo "  nu -c \"plugin use nw_ulid; ulid info\""