# Vero

Terminal email client in Rust with TUI and CLI workflows. Everything is a file.

## Install

```bash
cargo install --path .
```

## Configure

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

`editor` is used for compose and signatures.
`viewer` is used for opening emails from the TUI.
If either is missing, Vero falls back to `$EDITOR`.

## Run

```bash
vero
```

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

## TUI

- `j`/`k` or `↑`/`↓`: move
- `Enter`: open or select
- `Tab`: switch screens
- `Esc`: back
- `/`: search the current list
- `gg` / `G`: jump to top or bottom
- `q`: quit

### Inbox

- `r`: refresh
- `d`: delete
- `u` / `s` / `a`: unseen, seen, all
- `e`: open in `viewer` or `$EDITOR`

### List Search

- Works in inbox, sent, and drafts
- Inbox search works in all list filters: `all`, `unseen`, `seen`
- Matches subject plus sender/contact fields
- Filters live as you type

### Compose

The editor opens with:

```
to: recipient@example.com
cc:
bcc:
subject: Your subject
attachments: ~/file.pdf
body: Your message here
```

Required fields: `to`, `subject`

Compose and signature editing use `editor` or `$EDITOR`.

## Draft Format

TUI compose and `vero send --draft` use the same plain-text format:

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

- `src/cli/`: parsing, commands, output
- `src/services/`: shared mail operations
- `src/tui/`: runtime, handlers, UI
- `src/storage/`: local `.eml` persistence
- `src/email/imap_client/`: IMAP fetch and body parsing

## License

AGPL-3.0

---

Built with Rust • v2.0.0
