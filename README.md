# 📬 Vero

A terminal-based email client built in Rust that embraces the Unix philosophy. **Everything is a file.** Compose emails in your favorite editor, store them as plain text, and maintain full control over your email workflow.

## Philosophy

Vero follows the Unix philosophy:

- **Do one thing well**: Email client for power users
- **Everything is a file**: Emails, drafts, and sent messages are all human-readable text files
- **Composability**: Works with your favorite editor, not against it
- **Transparency**: No hidden formats - you see exactly what you're sending
- **Scriptability**: Draft files can be generated, templated, or version controlled

## Features

- **Editor-First Composition**: Write full emails (to, cc, bcc, subject, body) in your favorite editor
- **File-Based Storage**: All emails stored as plain text `.eml` files with a simple header-body format
- **Multiple Accounts**: Manage multiple email accounts with per-account organization
- **BCC Support**: Full support for blind carbon copy recipients
- **Attachment Support**: Add files via simple comma-separated paths in the email header
- **Inbox Management**: View and filter emails (unseen/seen/all) from any IMAP server
- **Sent Folder**: Browse locally stored sent emails
- **Auto-Refresh**: Optionally auto-refresh inbox at configurable intervals
- **Beautiful TUI**: Clean, colorful interface built with Ratatui
- **Keyboard-Driven**: Vim-style navigation with full keyboard control

## Prerequisites

- **Rust** 1.70 or higher
- **An IMAP/SMTP email account**
- **A terminal editor** (vim, nvim, nano, emacs, etc.) - **REQUIRED** for composing emails

## Installation

### Build from Source

```bash
git clone https://github.com/saravenpi/vero.git
cd vero
cargo build --release
sudo mv target/release/vero /usr/local/bin/
```

### Install with Cargo

```bash
cargo install --path .
```

## Configuration

Create a `~/.vero.yml` file:

```yaml
accounts:
  - email: your@example.com
    imap:
      password: your-password
      host: imap.example.com
      port: 993              # Optional, defaults to 993
      user: your@example.com # Optional, defaults to email
    smtp:
      password: your-password
      host: smtp.example.com
      port: 465              # Optional, defaults to 465
      user: your@example.com # Optional, defaults to email

  # Additional accounts
  - email: work@company.com
    imap:
      password: work-password
      host: imap.gmail.com
    smtp:
      password: work-password
      host: smtp.gmail.com

# REQUIRED: Editor for composing emails
editor: vim

# Optional settings
auto_refresh: 30          # Auto-refresh inbox every 30 seconds (or false/0 to disable)
inbox_view: all           # Default filter: unseen, seen, or all
viewer: bat               # External viewer for reading emails (optional)
```

### Editor Configuration (REQUIRED)

Vero **requires** an external editor - there is no fallback TUI editor. The `editor` field is **mandatory** in your configuration.

**Supported editors:**
- `vim`, `nvim`, `nano`, `emacs`, etc.
- Can include arguments: `editor: "nvim -c 'startinsert'"`

If you try to compose without an editor configured, Vero will show a helpful error message.

## Usage

### Starting Vero

```bash
vero              # Start the email client
vero version      # Show version
vero help         # Show help
```

### Navigation

**Account Selection** (if multiple accounts):
- `↑`/`↓` or `j`/`k` - Navigate accounts
- `Enter` - Select account

**Main Menu**:
- `↑`/`↓` or `j`/`k` - Navigate sections (Inbox, Sent, Compose)
- `Enter` - Enter section
- `Tab` - Switch focus to menu
- `q` - Quit

**Inbox**:
- `↑`/`↓` or `j`/`k` - Navigate emails
- `Enter` - View email
- `d` - Delete email (removes from local storage)
- `r` - Refresh inbox
- `u` - Filter: Unseen emails only
- `s` - Filter: Seen emails only
- `a` - Filter: All emails
- `e` - Open email in external viewer (if configured)
- `ESC` - Back to menu
- `q` - Quit

**Sent Folder**:
- `↑`/`↓` or `j`/`k` - Navigate sent emails
- `Enter` - View email
- `r` - Refresh
- `e` - Open in external viewer
- `ESC` - Back to menu

### Composing Emails

When you select "Compose", Vero creates a draft file and opens your editor:

**1. Draft File Creation**
A file is created at `~/.vero/<account>/drafts/<timestamp>.eml`:

```
to:
cc:
bcc:
subject:
attachments:
body:
```

**2. Edit in Your Editor**
Fill in the fields:

```
to: recipient@example.com
cc: colleague@example.com
bcc: boss@example.com
subject: Q4 Report
attachments: ~/Documents/report.pdf, ~/screenshot.png
body: Hi team,

Please find the Q4 report attached.

Best regards
```

**Key points:**
- `to` and `subject` are **required**
- `cc`, `bcc`, and `attachments` are **optional** (can be left empty or omitted)
- `from` is **not included** - automatically set from your account
- `body:` must be the **last field**
- Everything after `body:` is the email body
- Attachments: comma-separated file paths (supports `~` expansion)

**3. Save and Quit Editor**
Vero parses the draft and shows a preview.

**4. Preview Screen**
- `Enter` - Send email
- `e` - Edit again (reopens editor)
- `ESC` - Cancel (deletes draft)

### File Format Details

**Email headers** (order doesn't matter except `body:` must be last):
```
to: required@example.com
cc: optional@example.com, another@example.com
bcc: secret@example.com
subject: Required Subject
attachments: ~/file.pdf, /absolute/path/image.png
body: Everything after this line is the email body
It can span multiple lines
And preserve formatting
```

**Parsing rules:**
- Empty lines between headers are ignored
- Field names are case-insensitive
- Leading/trailing whitespace is trimmed
- `from:` field should NOT be in draft (it's auto-set)
- Unknown fields are ignored (forward compatibility)

## Data Storage

Vero organizes emails by account:

```
~/.vero/
├── your@example.com/
│   ├── drafts/
│   │   └── 2025-01-15-143022.eml      # Draft emails (temp)
│   ├── seen/
│   │   └── 2025-01-15-100000-sender_example_com.eml
│   └── sent/
│       └── 2025-01-15-120000-recipient_example_com.eml
└── work@company.com/
    ├── drafts/
    ├── seen/
    └── sent/
```

**File format:**
- **Extension**: `.eml` (not YAML anymore!)
- **Format**: Simple header-body text format
- **Human-readable**: Open any email file in your editor
- **Version control friendly**: Plain text, easy to diff

## Architecture

Built with Rust and modern async libraries:

- **[Ratatui](https://ratatui.rs)** - Terminal UI framework
- **[Tokio](https://tokio.rs)** - Async runtime
- **[async-imap](https://docs.rs/async-imap)** - IMAP client
- **[Lettre](https://lettre.rs)** - SMTP client
- **[mailparse](https://docs.rs/mailparse)** - Email parsing
- **[Crossterm](https://docs.rs/crossterm)** - Terminal manipulation

## Examples

### Quick Email

```bash
# Create a quick email (if you have templates set up)
cat > ~/.vero/user@example.com/drafts/quick.eml <<EOF
to: friend@example.com
subject: Quick question
body: Hey, just checking in!
EOF

# Then open Vero, go to Compose, and it'll parse your draft
```

### Email Templates

Since emails are just text files, you can create templates:

```bash
# Save template
cat > ~/email-template.eml <<EOF
to:
cc:
subject: Weekly Update
body: Hi team,

This week's update:

-
-
-

Best regards
EOF

# Use template
cp ~/email-template.eml ~/.vero/user@example.com/drafts/$(date +%Y-%m-%d-%H%M%S).eml
```

### Scripted Emails

```bash
# Generate email from script
cat > ~/.vero/user@example.com/drafts/$(date +%Y-%m-%d-%H%M%S).eml <<EOF
to: team@example.com
subject: Server Status Report $(date +%Y-%m-%d)
body: Daily server status:

$(uptime)

Disk usage:
$(df -h)
EOF
```

## Troubleshooting

### "No editor configured!"

Add `editor: vim` (or your preferred editor) to `~/.vero.yml`.

### "Draft parsing error"

Check that:
- `to:` and `subject:` fields are not empty
- `body:` is the last field
- All header lines have a colon `:`
- Attachment files exist and are readable

### "Connection failed"

- Verify IMAP/SMTP credentials in `~/.vero.yml`
- For Gmail: Use an [App Password](https://support.google.com/accounts/answer/185833)
- Check firewall/network settings

### "Attachment file not found"

- Use absolute paths or `~` for home directory
- Verify file exists: `ls ~/path/to/file.pdf`
- Check file permissions

## Migration from v1.x

**Breaking changes in v2.0:**
- **Storage format**: Changed from YAML to `.eml` text format
- **Directory structure**: Now per-account (`~/.vero/<account>/`)
- **Editor requirement**: Editor is now required (no TUI fallback)
- **Old data**: Not auto-migrated - clean break for v2.0

Old emails remain in `~/.vero/seen/` and `~/.vero/sent/` but won't be loaded by v2.0.

## Development

```bash
# Clone and build
git clone https://github.com/saravenpi/vero.git
cd vero
cargo build

# Run in development
cargo run

# Run tests
cargo test

# Build release
cargo build --release
```

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## License

AGPL-3.0

## Author

[saravenpi](https://github.com/saravenpi)

---

**Built with Rust 🦀**
**Version**: 2.0.0

*"Everything is a file, everything is composable."*
