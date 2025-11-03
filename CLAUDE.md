# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Vero is a terminal-based email client built in Go using Charmbracelet's TUI libraries (Bubble Tea, Bubbles, Lip Gloss). It provides inbox management, email composition, and sent folder views with IMAP/SMTP support.

## Development Commands

**Run the application**:
```bash
go run .
```

**Build the binary**:
```bash
go build -o vero .
```

**Run tests**:
```bash
go test ./...
```

**Install to PATH**:
```bash
go install
```

## Configuration

The application requires a `~/.vero.yml` configuration file with account settings:
- Each account needs `email`, `imap` (user, password, host, port), and `smtp` (user, password, host, port)
- User fields default to the email address if not provided
- Ports default to 993 (IMAP) and 465 (SMTP) if not specified
- Multiple accounts can be configured in the same file (see `internal/config/config.go`)
- Optional `editor` field to specify an external editor for composing email bodies (e.g., `neovim`, `vim`, `nano`)
  - If set, the specified editor will open in a temporary file when composing the email body (similar to git commit editor)
  - If not set, the built-in textarea editor is used
- Optional `auto_refresh` field to enable automatic email refreshing in the inbox:
  - Not set or `0`: No auto-refresh (default)
  - `false`: No auto-refresh
  - `true`: Auto-refresh every 10 seconds
  - Integer value (e.g., `30`): Auto-refresh every N seconds
  - When enabled, new emails are automatically fetched and merged into the list without disrupting the current view

## Architecture

The application follows the Bubble Tea architecture (Model-Update-View pattern):

**Entry Point** (`main.go`):
- Handles CLI arguments (version, help)
- Loads config and initializes the Bubble Tea program with MenuModel

**State Management** (`internal/models/types.go`):
- `Section` enum tracks navigation: Menu, Inbox, Sent, Compose
- `ViewMode` enum: List or Detail view
- `InboxFilter` enum: Unseen, Seen, or All emails
- `Email` struct is the core data model with YAML tags for persistence

**UI Models** (`internal/ui/`):
- Each view is a separate Bubble Tea model with Init(), Update(), View() methods
- `menu.go` - Main menu with section selection
- `inbox.go` - Email list and detail view with filtering (u/s/a keys)
- `compose.go` - Multi-step email composition (To → CC → Subject → Body → Preview), supports external editor via `tea.Exec` or built-in textarea
- `sent.go` - Sent emails viewer
- `styles.go` - Centralized Lip Gloss styling

**Email Operations**:
- `internal/email/imap.go` - IMAP client for fetching emails
- `internal/email/smtp.go` - SMTP client for sending emails
- `internal/email/fetch_body.go` - Email body parsing logic

**Data Persistence** (`internal/storage/storage.go`):
- Emails stored as YAML files in `~/.vero/seen/` and `~/.vero/sent/`
- Filename format: `YYYY-MM-DD-HHMMSS-email@address.yml`

## Key Design Patterns

**Bubble Tea Model Switching**: The main menu model (`ui/menu.go`) manages child models (inbox, sent, compose) and switches between them based on user navigation. When a section is active, the menu model delegates Init(), Update(), and View() calls to the active child model.

**Email State**: Emails are marked as "seen" by saving them to `~/.vero/seen/` when viewed. The inbox filter system allows toggling between unseen, seen, and all emails.

**Configuration Defaults**: User fields default to the account email if not provided, and ports default to 993 (IMAP) and 465 (SMTP) if not specified, allowing minimal configuration.

## Dependencies

- `github.com/charmbracelet/bubbletea` - TUI framework
- `github.com/charmbracelet/bubbles` - Text input/textarea components
- `github.com/charmbracelet/lipgloss` - Terminal styling
- `github.com/emersion/go-imap` - IMAP protocol
- `github.com/emersion/go-smtp` - SMTP protocol
- `gopkg.in/yaml.v3` - YAML serialization

## Version

Current version: 2.0.0 (defined in `main.go:12`)
