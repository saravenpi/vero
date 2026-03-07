# Vero

Terminal email client built in Rust. Everything is a file.

## Philosophy

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
auto_refresh: 30
inbox_view: all
```

Required: `editor` field (vim, nvim, nano, emacs, etc.)

## Usage

```bash
vero
```

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

## Storage

```
~/.vero/<account>/
├── drafts/
├── seen/
└── sent/
```

All files are human-readable `.eml` format.

## License

AGPL-3.0

---

Built with Rust • v2.0.0
