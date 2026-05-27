#!/usr/bin/env bash
set -e

REPO="saravenpi/vero"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="vero"

echo "🚀 Installing Vero email client..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo is not installed. Please install Rust first: https://rustup.rs/"
    exit 1
fi

echo "✓ Cargo found: $(cargo --version)"

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

SKILL_MARKER_START="<!-- vero:start -->"
SKILL_MARKER_END="<!-- vero:end -->"

inject_block() {
  local file="$1"
  local content="$2"
  local block
  block="$(printf '%s\n%s\n%s' "$SKILL_MARKER_START" "$content" "$SKILL_MARKER_END")"

  if [ ! -f "$file" ]; then
    printf '%s\n' "$block" > "$file"
    return
  fi

  if grep -qF "$SKILL_MARKER_START" "$file"; then
    local tmp
    tmp="$(mktemp)"
    awk -v start="$SKILL_MARKER_START" -v end="$SKILL_MARKER_END" '
      $0 == start { skip=1; next }
      $0 == end   { skip=0; next }
      !skip       { print }
    ' "$file" > "$tmp"
    mv "$tmp" "$file"
    printf '\n%s\n' "$block" >> "$file"
  else
    printf '\n%s\n' "$block" >> "$file"
  fi
}

register_skill() {
  local repo_dir="$1"
  local skill_file="$repo_dir/integrations/SKILL.md"

  [ -f "$skill_file" ] || return 0

  local skill_content
  skill_content="$(cat "$skill_file")"

  if command -v claude &>/dev/null; then
    local skill_dir="$HOME/.claude/skills/vero"
    mkdir -p "$skill_dir"
    cp "$skill_file" "$skill_dir/SKILL.md"
    inject_block "$HOME/.claude/CLAUDE.md" "$skill_content"
    echo "  ✓ Claude Code skill registered"
  fi

  if command -v codex &>/dev/null; then
    mkdir -p "$HOME/.codex"
    inject_block "$HOME/.codex/AGENTS.md" "$skill_content"
    echo "  ✓ Codex skill registered"
  fi
}

cd "$TEMP_DIR"

echo "⬇️  Downloading Vero..."
if command -v git &> /dev/null; then
    git clone --depth 1 "https://github.com/$REPO.git" vero
    cd vero
else
    curl -fsSL "https://github.com/$REPO/archive/refs/heads/master.tar.gz" | tar xz
    cd vero-master
fi

REPO_DIR="$(pwd)"

echo "🔨 Building Vero..."
cargo build --release

echo "📥 Installing to $INSTALL_DIR/$BINARY_NAME..."
mv "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

register_skill "$REPO_DIR"

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
