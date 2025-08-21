#!/bin/bash

# Plugin installation script
# Builds and installs the plugin for Nushell

set -e

echo "🔌 Installing nu_plugin_nw_ulid for Nushell..."

# Source cargo environment if it exists
if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
fi

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Error: This script must be run from the project root directory"
    exit 1
fi

# Check if Nushell is installed
if ! command -v nu &> /dev/null; then
    echo "❌ Nushell is not installed or not in PATH"
    echo "   Please install Nushell first:"
    echo "   cargo install nu"
    exit 1
fi

echo "✅ Nushell found: $(nu --version)"

# Parse command line arguments
INSTALL_PATH=""
FORCE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --path)
            INSTALL_PATH="$2"
            shift 2
            ;;
        --force)
            FORCE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --path PATH   Install to specific path (default: ~/.cargo/bin)"
            echo "  --force       Force reinstallation if already installed"
            echo "  -h, --help    Show this help message"
            echo ""
            echo "This script will:"
            echo "  1. Build the plugin in release mode"
            echo "  2. Install the plugin binary"
            echo "  3. Register the plugin with Nushell"
            echo "  4. Test the plugin installation"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Determine installation path
if [[ -z "$INSTALL_PATH" ]]; then
    INSTALL_PATH="$HOME/.cargo/bin/nu_plugin_nw_ulid"
fi

echo "📋 Installation Configuration:"
echo "   Plugin path: $INSTALL_PATH"
echo "   Force reinstall: $FORCE"
echo ""

# Check if plugin is already installed
if [[ -f "$INSTALL_PATH" ]] && [[ "$FORCE" == "false" ]]; then
    echo "⚠️  Plugin already installed at: $INSTALL_PATH"
    echo "   Use --force to reinstall"
    echo "   Or remove manually: rm $INSTALL_PATH"
    exit 1
fi

# Build the plugin
echo "🔨 Building plugin in release mode..."
if ! cargo build --release; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Plugin built successfully"

# Install the plugin
echo "📦 Installing plugin..."
if [[ "$INSTALL_PATH" == "$HOME/.cargo/bin/nu_plugin_nw_ulid" ]]; then
    # Use cargo install for standard location
    cargo install --path . --force
    echo "✅ Plugin installed via cargo install"
else
    # Copy to custom location
    cp target/release/nu_plugin_nw_ulid "$INSTALL_PATH"
    chmod +x "$INSTALL_PATH"
    echo "✅ Plugin copied to: $INSTALL_PATH"
fi

# Verify binary
if [[ ! -f "$INSTALL_PATH" ]]; then
    echo "❌ Plugin binary not found after installation"
    exit 1
fi

echo "✅ Plugin binary verified"

# Test the plugin binary
echo "🧪 Testing plugin binary..."
if "$INSTALL_PATH" --help &> /dev/null; then
    echo "✅ Plugin binary responds correctly"
else
    echo "⚠️  Plugin binary test had issues (this might be expected for Nushell plugins)"
fi

# Register with Nushell
echo "🔌 Registering plugin with Nushell..."

# Create a temporary Nushell script to register the plugin
TEMP_SCRIPT=$(mktemp)
cat > "$TEMP_SCRIPT" << EOF
plugin add $INSTALL_PATH
plugin use nw_ulid
ulid info
EOF

echo "   Running: nu $TEMP_SCRIPT"
if nu "$TEMP_SCRIPT"; then
    echo "✅ Plugin registered and tested successfully"
    REGISTRATION_SUCCESS=true
else
    echo "⚠️  Plugin registration had issues"
    REGISTRATION_SUCCESS=false
fi

# Clean up
rm "$TEMP_SCRIPT"

# Final verification
echo ""
echo "🔍 Final verification..."

# Check if plugin shows up in Nushell
VERIFY_SCRIPT=$(mktemp)
cat > "$VERIFY_SCRIPT" << EOF
plugin list | where name == "ulid"
EOF

echo "   Checking plugin list..."
if nu "$VERIFY_SCRIPT" | grep -q "ulid"; then
    echo "✅ Plugin appears in Nushell plugin list"
else
    echo "⚠️  Plugin not found in Nushell plugin list"
fi

rm "$VERIFY_SCRIPT"

echo ""
echo "🎉 Plugin installation completed!"
echo ""
echo "📋 Installation Summary:"
echo "   Plugin binary: $INSTALL_PATH"
echo "   Binary size: $(ls -lh "$INSTALL_PATH" | awk '{print $5}')"
echo "   Nushell registration: $($REGISTRATION_SUCCESS && echo 'Success' || echo 'Needs manual setup')"
echo ""
echo "🚀 Usage:"
echo "   Start Nushell: nu"
echo "   Check plugin: ulid info"
echo "   Get help: help ulid"
echo ""
echo "🔧 Manual registration (if needed):"
echo "   plugin add $INSTALL_PATH"
echo "   plugin use nw_ulid"
echo ""
echo "📚 Documentation:"
echo "   README.md - Usage examples and commands"
echo "   docs/ - Detailed documentation"
