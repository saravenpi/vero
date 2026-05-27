---
name: vero
description: >
  Terminal email client with CLI and TUI. Use when the user asks to read,
  send, draft, triage, or manage emails.
---

# vero — Terminal email client

Binary: `vero`
Config: `~/.vero.yml`

## When to apply

Use when the user mentions email, inbox, sending email, drafting, unread mail, or email management.
Triggers: "check email", "send an email", "inbox", "unread", "draft", "reply", "sent mail", "email"

## Commands

### Accounts
```
vero accounts                  List configured accounts
```

### Inbox
```
vero --account <acct> inbox list [--filter unseen|seen|all] [--limit <n>]
vero --account <acct> inbox show <uid>
vero --account <acct> inbox delete <uid>
vero --account <acct> inbox unread-count
vero --account <acct> inbox download <uid> [--index <n>]
```

### Sent
```
vero --account <acct> sent list [--limit <n>]
vero --account <acct> sent show <index>     1-based index
```

### Send
```
vero --account <acct> send --to <addr> --subject <subj> --body <text>
vero --account <acct> send --to <addr> --subject <subj> --cc <addr> --bcc <addr> --attach <path>
vero --account <acct> send --draft <path|->
```

### Drafts
```
vero draft template [--output <path>]
```

### Other
```
vero tui                       Launch interactive TUI
vero version
vero help
```

## Rules
- `--account` accepts exact email or 1-based index from `vero accounts`
- `--json` available on most commands
- No dedicated reply command — read original, then send with appropriate subject/body
- Draft format: plain text headers (to, subject, cc, bcc, attachments) then `body:` last
- Confirm before sending or deleting
- Resolve account with `vero accounts` first if multiple configured
- After using vero, tell the user what was read, found, drafted, sent, or deleted
