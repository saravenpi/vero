#!/usr/bin/env bash
set -e

REPO="saravenpi/vero"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="vero"

echo "🚀 Installing Vero email client..."

if ! command -v go &> /dev/null; then
    echo "❌ Error: Go is not installed. Please install Go first: https://golang.org/dl/"
    exit 1
fi

echo "✓ Go found: $(go version)"

if [ ! -d "$INSTALL_DIR" ]; then
    echo "📁 Creating $INSTALL_DIR..."
    mkdir -p "$INSTALL_DIR"
fi

TEMP_DIR=$(mktemp -d)
echo "📦 Using temporary directory: $TEMP_DIR"

cleanup() {
    echo "🧹 Cleaning up temporary files..."
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

cd "$TEMP_DIR"

echo "⬇️  Downloading Vero..."
if command -v git &> /dev/null; then
    git clone --depth 1 "https://github.com/$REPO.git" vero
    cd vero
else
    curl -fsSL "https://github.com/$REPO/archive/refs/heads/master.tar.gz" | tar xz
    cd vero-master
fi

echo "🔨 Building Vero..."
go build -o "$BINARY_NAME" .

echo "📥 Installing to $INSTALL_DIR/$BINARY_NAME..."
mv "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo ""
echo "✅ Vero installed successfully!"
echo ""
echo "📋 Next steps:"
echo "  1. Ensure $INSTALL_DIR is in your PATH"
echo "     Add this to your ~/.bashrc or ~/.zshrc:"
echo "     export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
echo "  2. Create configuration file at ~/.vero.yml"
echo "     See: https://github.com/$REPO#configuration"
echo ""
echo "  3. Run: vero"
echo ""

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Warning: $INSTALL_DIR is not in your PATH"
    echo "   Run: export PATH=\"\$HOME/.local/bin:\$PATH\""
fi
