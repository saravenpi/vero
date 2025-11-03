package main

import (
	"fmt"
	"os"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/ui"
)

const version = "2.0.0"

func main() {
	if len(os.Args) > 1 {
		switch os.Args[1] {
		case "version", "-v", "--version":
			fmt.Printf("Vero v%s (Go)\n", version)
			return
		case "help", "-h", "--help":
			printHelp()
			return
		default:
			fmt.Printf("Unknown command: %s\n", os.Args[1])
			printHelp()
			os.Exit(1)
		}
	}

	cfg, err := config.Load()
	if err != nil {
		fmt.Printf("Error loading config: %v\n", err)
		os.Exit(1)
	}

	var initialModel tea.Model
	if len(cfg.Accounts) == 1 {
		initialModel = ui.NewMenuModel(cfg, &cfg.Accounts[0])
	} else {
		initialModel = ui.NewAccountSelectionModel(cfg)
	}

	p := tea.NewProgram(initialModel, tea.WithAltScreen())
	if _, err := p.Run(); err != nil {
		fmt.Printf("Error: %v\n", err)
		os.Exit(1)
	}
}

func printHelp() {
	help := `Vero - Terminal Email Client

Usage:
  vero              Start the email client
  vero version      Show version information
  vero help         Show this help message

Configuration:
  Create a ~/.vero.yml file with your accounts:

  # Global settings (optional)
  download_folder: ~/Downloads          # Default: ~/Downloads
  inbox_view: all                        # Options: unseen, seen, all (Default: all)

  # Account configuration
  accounts:
    - email: your@email.com
      imap:
        user: your@email.com             # Optional, defaults to email
        password: your-password
        host: imap.example.com
        port: 993                         # Optional, defaults to 993
      smtp:
        user: your@email.com             # Optional, defaults to email
        password: your-password
        host: smtp.example.com
        port: 465                         # Optional, defaults to 465

  You can configure multiple accounts in the same file.

Navigation:
  ↑/↓ or j/k        Navigate lists
  Enter             Select/Open
  ESC               Go back
  q                 Quit

Inbox Filters:
  u                 Show unseen emails only
  s                 Show seen emails only
  a                 Show all emails

Compose:
  Tab               Autocomplete file paths
  Ctrl+D            Finish writing body and preview
  Enter             Confirm and move to next field

Attachments:
  o                 Open attachment with default app
  d                 Download attachment to configured folder
  ←→ or h/l         Navigate between attachments

Data Storage:
  Emails are stored locally in ~/.vero/
    <account-email>/seen/        Viewed emails for each account
    <account-email>/sent/        Sent emails for each account
    attachments/                 Downloaded attachments
`
	fmt.Print(help)
}
