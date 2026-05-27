#!/bin/bash
set -euo pipefail

REPO="https://github.com/saravenpi/vero.git"
BIN_NAME="vero"

info()  { printf '\033[1;36m%s\033[0m\n' "$*"; }
error() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

command -v cargo >/dev/null 2>&1 || error "cargo not found. Install Rust first: https://rustup.rs"
command -v git   >/dev/null 2>&1 || error "git not found. Install git first."

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

info "Cloning $REPO..."
git clone --depth 1 --quiet "$REPO" "$TMPDIR/vero"

info "Building and installing $BIN_NAME..."
cargo install --path "$TMPDIR/vero" --force --quiet

# --- AI agent skill registration ---
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
    info "  ✓ Claude Code skill registered"
  fi

  if command -v codex &>/dev/null; then
    mkdir -p "$HOME/.codex"
    inject_block "$HOME/.codex/AGENTS.md" "$skill_content"
    info "  ✓ Codex skill registered"
  fi
}

register_skill "$TMPDIR/vero"

INSTALL_PATH="$(command -v "$BIN_NAME" 2>/dev/null || echo "$HOME/.cargo/bin/$BIN_NAME")"
info "Installed $BIN_NAME to $INSTALL_PATH"
