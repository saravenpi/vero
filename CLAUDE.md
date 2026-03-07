# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Vero is a terminal-based email client built in Rust using Ratatui. It embraces the Unix philosophy by providing a file-based, editor-first approach to email composition. Every email is a file, making the workflow powerful, transparent, and scriptable.

## Development Commands

**Run the application**:
```bash
cargo run
```

**Build the binary**:
```bash
cargo build --release
```

**Run tests**:
```bash
cargo test
```

**Install to PATH**:
```bash
cargo install --path .
```

## Configuration

The application requires a `~/.vero.yml` configuration file with account settings:

**Required fields:**
- Each account needs `email`, `imap` (user, password, host, port), and `smtp` (user, password, host, port)
- User fields default to the email address if not provided
- Ports default to 993 (IMAP) and 465 (SMTP) if not specified
- Multiple accounts can be configured in the same file

**Editor configuration (REQUIRED for composing emails):**
- `editor` field specifies the external editor for composing emails (e.g., `vim`, `nvim`, `nano`, `emacs`)
- The editor is **required** - there is no fallback TUI editor
- Supports arguments: `editor: "nvim -c 'startinsert'"`
- The editor opens a draft file in `~/.vero/<account>/drafts/` with the full email structure

**Optional fields:**
- `auto_refresh`: Enable automatic email refreshing in the inbox
  - Not set or `0`: No auto-refresh (default)
  - `false`: No auto-refresh
  - `true`: Auto-refresh every 10 seconds
  - Integer value (e.g., `30`): Auto-refresh every N seconds
- `viewer`: External viewer for reading emails (e.g., `less`, `bat`)

**Example configuration:**
```yaml
accounts:
  - email: user@example.com
    imap:
      password: secret
      host: imap.example.com
    smtp:
      password: secret
      host: smtp.example.com
editor: vim
auto_refresh: 30
```

## Architecture

Vero follows the Ratatui TUI architecture with a file-based email system:

**Entry Point** (`src/main.rs`):
- Handles CLI arguments (version, help)
- Loads config and initializes the Ratatui terminal
- Main event loop handling keyboard input and async operations

**State Management** (`src/tui/app.rs`):
- `Screen` enum tracks navigation: AccountSelection, Inbox, Sent, Compose
- `ViewMode` enum: List or Detail view
- `InboxFilter` enum: Unseen, Seen, or All emails
- `ComposeStep` enum: Editing, Preview, NoEditor

**Models** (`src/models/mod.rs`):
- `Email` struct: Core email data model with serde serialization
- `EmailDraft` struct: Draft email with to, cc, bcc, subject, body, attachments
- `Attachment` struct: File attachment metadata

**Email Operations**:
- `src/email/imap_client.rs` - IMAP client for fetching emails
- `src/email/smtp_client.rs` - SMTP client for sending emails (supports CC, BCC, attachments)
- `src/email_file.rs` - Email file format parser/writer

**Email File Format** (`src/email_file.rs`):
All emails (drafts, sent, seen) use a simple, human-readable header-body format:
```
to: recipient@example.com
cc: another@example.com
bcc: secret@example.com
subject: Email subject
attachments: ~/file.pdf, /path/to/image.png
body: Email body starts here
and continues on multiple lines...
```

Key points:
- Headers first, `body:` must be last
- `from:` field is NOT in draft files - automatically set from account
- Required fields: to, subject
- Optional fields: cc, bcc, attachments
- Everything after `body:` is the email body

**Data Persistence** (`src/storage/mod.rs`):
- **Per-account directory structure**: `~/.vero/<account>/drafts/`, `~/.vero/<account>/sent/`, `~/.vero/<account>/seen/`
- Emails stored as `.eml` files using the email file format (not YAML)
- Filename format: `YYYY-MM-DD-HHMMSS.eml` for drafts, `YYYY-MM-DD-HHMMSS-sanitized_email.eml` for sent/seen
- Draft files are created when composing, deleted after sending or canceling

**UI Rendering** (`src/tui/ui.rs`):
- Ratatui-based terminal UI with colored, bordered panels
- No TUI form inputs for compose - editor-only workflow
- Preview screen shows parsed email before sending

## Key Design Patterns

**Editor-First Composition**:
1. User selects "Compose" from menu
2. If no editor configured, show helpful error and return to menu
3. Create draft file at `~/.vero/<account>/drafts/<timestamp>.eml`
4. Open editor with draft file
5. Parse draft file after editor closes
6. Show preview screen
7. User can send (Enter), edit again ('e'), or cancel (ESC)

**File-Based Architecture**:
- Everything is a file - emails, drafts, sent messages
- Users can edit drafts with any editor, not just in the TUI
- Drafts can be scripted, version controlled, templated
- Transparent - users see exactly what they're sending

**Unix Philosophy**:
- Do one thing well: email client
- Plain text files for data storage
- Composable with other tools (editors, viewers)
- No hidden formats - all email files are human-readable

## Dependencies

- `tokio` - Async runtime
- `ratatui` - TUI framework
- `crossterm` - Terminal manipulation
- `serde`, `serde_yaml` - Serialization
- `anyhow` - Error handling
- `chrono` - Date/time handling
- `async-imap` - IMAP protocol
- `lettre` - SMTP protocol
- `mailparse` - Email parsing
- `dirs` - Directory paths

## Version

Current version: 2.0.0

## Migration from v1.x

**Breaking changes in v2.0:**
- Storage moved from flat `~/.vero/seen/` and `~/.vero/sent/` to per-account directories
- Email format changed from YAML to `.eml` format
- Compose now requires `editor` in config (no TUI fallback)
- Old emails are NOT automatically migrated - clean break for v2.0

## Testing Email Composition

To test the new email composition flow:
1. Ensure `editor` is set in `~/.vero.yml`
2. Select "Compose" from menu
3. Editor opens with template file
4. Fill in to, subject, body (cc, bcc, attachments optional)
5. Save and quit editor
6. Preview screen shows parsed email
7. Press Enter to send, 'e' to edit again, ESC to cancel
