# Vero

A fast, beautiful terminal-based email client built with Go and [Charmbracelet](https://github.com/charmbracelet) TUI libraries. Read your emails and compose new ones directly from your terminal with an intuitive interface.

## Features

- **Multiple Accounts**: Manage multiple email accounts from a single interface
- **Inbox Management**: View and read emails from any IMAP-compatible server
- **Email Composition**: Write and send emails with an intuitive TUI
- **Sent Folder**: Browse your locally stored sent emails
- **YAML Database**: Emails are stored locally as YAML files for easy access
- **Beautiful TUI**: Built with Bubble Tea, Bubbles, and Lip Gloss
- **Keyboard Navigation**: Full keyboard control with vim-style bindings
- **IMAP/SMTP Support**: Works with any email provider that supports IMAP and SMTP

## Prerequisites

- Go 1.23 or higher
- An IMAP/SMTP email account

## Installation

### Quick Install

Install Vero with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/saravenpi/vero/master/install.sh | bash
```

This will install the binary to `~/.local/bin/vero`. Make sure `~/.local/bin` is in your PATH.

### Build from Source

```bash
git clone https://github.com/saravenpi/vero.git
cd vero
go build -o vero .
```

### Manual Install

Move the binary to your PATH:

```bash
sudo mv vero /usr/local/bin/
```

Or use `go install`:

```bash
go install
```

### Configuration

Create a `~/.vero.yml` file in your home directory with your email accounts:

```yaml
accounts:
  - email: your@email.com
    imap:
      user: your@email.com      # Optional, defaults to email
      password: your-password
      host: imap.example.com
      port: 993                  # Optional, defaults to 993
    smtp:
      user: your@email.com      # Optional, defaults to email
      password: your-password
      host: smtp.example.com
      port: 465                  # Optional, defaults to 465

  # Add more accounts as needed
  - email: work@company.com
    imap:
      password: work-password
      host: imap.gmail.com
    smtp:
      password: work-password
      host: smtp.gmail.com
```

**Note**: The `user` and `port` fields are optional. If not specified, `user` defaults to the account email and ports default to 993 (IMAP) and 465 (SMTP).

## Usage

### Commands

```bash
vero              # Start the email client
vero version      # Show version information
vero help         # Show help message
```

### Account Selection

If you have multiple accounts configured, Vero will first show an account selection screen. Choose which account to use, then access its sections.

### Main Menu

After selecting an account (or if you only have one account), the main menu shows three sections:
- **Inbox**: View your emails
- **Sent**: Browse sent emails stored locally
- **Write**: Compose and send a new email

Use arrow keys (↑/↓) or vim keys (j/k) to navigate, Enter to select.

### Inbox

- **Navigate**: Use ↑/↓ or j/k to move through the email list
- **View Email**: Press Enter to read the selected email
- **Filter Emails**:
  - `u` - Show unseen emails only
  - `s` - Show seen emails only
  - `a` - Show all emails
- **Back to List**: Press ESC to return from email details
- **Back to Menu**: Press ESC from the list
- **Quit**: Press q or Ctrl+C

### Write Email

Follow the interactive prompts:
1. **To**: Enter recipient email address (required)
2. **CC**: Enter CC recipients (optional)
3. **Subject**: Enter email subject (required)
4. **Body**: Type your message (press Ctrl+D to finish)
5. **Preview**: Review your email
6. **Send**: Press Enter to send or ESC to edit

### Sent Folder

View all emails you've sent, stored locally as YAML files in `~/.vero/sent/`.

## Data Storage

Vero stores emails locally in `~/.vero/`, organized by account:

```
~/.vero/
├── your@email.com/
│   ├── seen/         # Emails you've viewed
│   │   └── YYYY-MM-DD-HHMMSS-sender@email.yml
│   └── sent/         # Emails you've sent
│       └── YYYY-MM-DD-HHMMSS-recipient@email.yml
└── work@company.com/
    ├── seen/
    └── sent/
```

Each email is stored as a YAML file with the following structure:

```yaml
from: sender@example.com
subject: Email Subject
date: Mon, 01 Nov 2025 12:34:56 +0000
body: Email content...
timestamp: '2025-11-01T12:34:56.000Z'
```

## Architecture

Vero is built with:

- [Bubble Tea](https://github.com/charmbracelet/bubbletea) - TUI framework based on The Elm Architecture
- [Bubbles](https://github.com/charmbracelet/bubbles) - TUI components (text inputs, text areas)
- [Lip Gloss](https://github.com/charmbracelet/lipgloss) - Terminal styling and layouts
- [go-imap](https://github.com/emersion/go-imap) - IMAP client library
- [go-smtp](https://github.com/emersion/go-smtp) - SMTP client library
- [go-yaml](https://gopkg.in/yaml.v3) - YAML parser for local storage

**Note**: For Gmail, you'll need to use an [App Password](https://support.google.com/accounts/answer/185833).

## Project Structure

```
vero/
├── main.go                          # Entry point
├── internal/
│   ├── config/
│   │   └── config.go               # Configuration loading (YAML)
│   ├── email/
│   │   ├── imap.go                 # IMAP client
│   │   └── smtp.go                 # SMTP client
│   ├── models/
│   │   └── types.go                # Data types
│   ├── storage/
│   │   └── storage.go              # YAML storage
│   └── ui/
│       ├── account_selection.go   # Account selection view
│       ├── compose.go              # Email composition view
│       ├── inbox.go                # Inbox view
│       ├── menu.go                 # Main menu
│       ├── sent.go                 # Sent emails view
│       └── styles.go               # Lip Gloss styles
├── .vero.example.yml                # Example configuration
├── go.mod
├── go.sum
└── README.md
```

## Development

Clone the repository:

```bash
git clone https://github.com/saravenpi/vero.git
cd vero
```

Create a `~/.vero.yml` file with your accounts (see Configuration section).

Run in development:

```bash
go run .
```

Build:

```bash
go build -o vero .
```

Test:

```bash
go test ./...
```

## Troubleshooting

### "Command not found: vero"

Make sure the binary is in your PATH. You can either:
- Move it to `/usr/local/bin/`: `sudo mv vero /usr/local/bin/`
- Or add the current directory to PATH

### "Connection failed"

- Check your email credentials in `~/.vero.yml`
- Ensure your email provider allows IMAP/SMTP access
- For Gmail, use an App Password instead of your regular password
- Verify the IMAP/SMTP host and port are correct

### "No emails found"

By default, Vero shows unseen emails. Press:
- `u` for unseen emails
- `s` for seen emails
- `a` for all emails

## Uninstall

To completely remove Vero from your system:

```bash
rm -rf ~/.vero
rm ~/.local/bin/vero  # if installed with install.sh
# or
sudo rm /usr/local/bin/vero  # if installed manually
```

This will remove the binary and all stored emails/configuration.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT

## Author

[saravenpi](https://github.com/saravenpi)

---

Built with [Go](https://go.dev) and [Charmbracelet](https://github.com/charmbracelet)

**Version**: 2.0.0
