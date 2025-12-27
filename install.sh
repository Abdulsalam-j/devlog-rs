#!/bin/bash

# DevLog Installation Script
# This script builds and installs devlog globally

set -e

echo "🚀 Installing DevLog..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build and install
echo "📦 Building DevLog..."
cargo build --release

echo "🔧 Installing globally..."
cargo install --path . --force

# Check if installation was successful
if command -v devlog &> /dev/null; then
    echo "✅ DevLog installed successfully!"
    echo ""
    echo "📝 Next steps:"
    echo "   1. Create config: cp devlog.toml ~/.config/devlog/devlog.toml"
    echo "   2. Edit config with your settings"
    echo "   3. Set up rclone: rclone config"
    echo "   4. Run: devlog log"
    echo ""
    echo "Run 'devlog --help' for more info!"
else
    echo "⚠️  Installation completed, but 'devlog' command not found in PATH"
    echo "   Make sure ~/.cargo/bin is in your PATH"
    echo "   Add this to your ~/.zshrc or ~/.bashrc:"
    echo "   export PATH=\"\$HOME/.cargo/bin:\$PATH\""
fi
