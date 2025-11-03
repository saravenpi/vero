package ui

import (
	"fmt"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/email"
	"github.com/saravenpi/vero/internal/models"
)

type unseenCountMsg struct {
	count int
	err   error
}

type MenuModel struct {
	cfg         *config.VeroConfig
	account     *config.Account
	choices     []string
	cursor      int
	selected    models.Section
	unseenCount int
	loading     bool
}

// NewMenuModel creates a new menu model for the specified account.
func NewMenuModel(cfg *config.VeroConfig, account *config.Account) MenuModel {
	return MenuModel{
		cfg:         cfg,
		account:     account,
		choices:     []string{"Inbox", "Sent", "Write"},
		cursor:      0,
		unseenCount: -1,
		loading:     true,
	}
}

func (m MenuModel) Init() tea.Cmd {
	return m.fetchUnseenCountCmd()
}

func (m MenuModel) fetchUnseenCountCmd() tea.Cmd {
	return func() tea.Msg {
		emails, err := email.FetchEmails(&m.account.IMAP, models.FilterUnseen)
		if err != nil {
			return unseenCountMsg{count: 0, err: err}
		}
		return unseenCountMsg{count: len(emails), err: nil}
	}
}

func (m MenuModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case unseenCountMsg:
		m.loading = false
		if msg.err == nil {
			m.unseenCount = msg.count
		}
		return m, nil

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			return m, tea.Quit

		case "esc":
			if len(m.cfg.Accounts) > 1 {
				accountSelection := NewAccountSelectionModel(m.cfg)
				return accountSelection, accountSelection.Init()
			}

		case "up", "k":
			if m.cursor > 0 {
				m.cursor--
			}

		case "down", "j":
			if m.cursor < len(m.choices)-1 {
				m.cursor++
			}

		case "enter":
			switch m.cursor {
			case 0:
				inbox := NewInboxModel(m.cfg, m.account)
				return inbox, inbox.Init()
			case 1:
				sent := NewSentModel(m.cfg, m.account)
				return sent, sent.Init()
			case 2:
				compose := NewComposeModel(m.cfg, m.account)
				return compose, compose.Init()
			}
		}
	}

	return m, nil
}

func (m MenuModel) View() string {
	s := titleStyle.Render(fmt.Sprintf("Vero - %s", m.account.Email)) + "\n\n"

	for i, choice := range m.choices {
		cursor := " "
		displayChoice := choice

		if i == 0 && m.unseenCount >= 0 {
			displayChoice = fmt.Sprintf("%s (%d)", choice, m.unseenCount)
		}

		if m.cursor == i {
			cursor = ">"
			displayChoice = selectedStyle.Render(displayChoice)
		} else {
			displayChoice = normalStyle.Render(displayChoice)
		}
		s += fmt.Sprintf("  %s %s\n", cursor, displayChoice)
	}

	helpText := "↑/↓ or j/k: navigate • enter: select"
	if len(m.cfg.Accounts) > 1 {
		helpText += " • esc: accounts"
	}
	helpText += " • q: quit"
	s += "\n" + helpStyle.Render(helpText)

	return s
}
