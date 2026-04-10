# Vero

Terminal email client built in Rust with both TUI and CLI workflows. Everything is a file.

## Philosophy

- Shared core email operations for TUI and CLI
- Editor-first composition workflow
- File-based storage (plain text `.eml` files)
- Unix composability
- Zero hidden formats

## Installation

```bash
cargo install --path .
```

## Configuration

Create `~/.vero.yml`:

```yaml
accounts:
  - email: you@example.com
    imap:
      password: your-password
      host: imap.example.com
    smtp:
      password: your-password
      host: smtp.example.com

editor: vim
viewer: less
auto_refresh: 30
inbox_view: all
```

`editor` is only required if you want TUI compose support.

## Usage

```bash
vero
```

Starts the TUI.

### CLI

```bash
vero [--account <email-or-index>] [--json] <command>
```

Useful commands:

- `vero accounts`
- `vero --account work inbox list --filter unseen`
- `vero --account work inbox show 4242`
- `vero --account work inbox delete 4242`
- `vero --account work inbox unread-count`
- `vero --account work sent list --limit 20`
- `vero --account work sent show 1`
- `vero --account work send --to alice@example.com --subject "Ping" --body "Hi"`
- `vero --account work send --draft draft.eml`
- `cat draft.eml | vero --account work send --draft -`
- `vero draft template --output draft.eml`

Use `--json` when another tool or model should parse the output instead of eyeballing it.

### Navigation

- `j`/`k` or `↑`/`↓` - Navigate
- `Enter` - Select
- `Tab` - Switch focus
- `ESC` - Back
- `q` - Quit

### Inbox

- `r` - Refresh
- `d` - Delete
- `u`/`s`/`a` - Filter (unseen/seen/all)
- `e` - Open in external viewer
- `Enter` - View email

### Compose

Opens your editor with:

```
to: recipient@example.com
cc:
bcc:
subject: Your subject
attachments: ~/file.pdf
body: Your message here
```

Required: `to` and `subject`

Preview → `Enter` to send, `e` to edit, `ESC` to cancel

### Draft File Format

The TUI compose flow and `vero send --draft` use the same plain-text format:

```
to: recipient@example.com
cc:
bcc:
subject: Your subject
attachments: ~/file.pdf
body: Your message here
```

## Storage

```
~/.vero/<account>/
├── drafts/
├── inbox/
├── seen/
└── sent/
```

All files are human-readable `.eml` format.

## Layout

The project is split so the TUI and CLI share the same service layer:

- `src/app/` top-level app entry and mode dispatch
- `src/cli/` command parsing, execution, help, and output formatting
- `src/services/` account resolution, inbox, sent mail, draft parsing, and sending
- `src/tui/` runtime, handlers, and UI rendering
- `src/storage/` local `.eml` persistence

## License

AGPL-3.0

---

Built with Rust • v2.0.0
