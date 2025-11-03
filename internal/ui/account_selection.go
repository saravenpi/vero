package ui

import (
	"fmt"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/email"
	"github.com/saravenpi/vero/internal/models"
)

type accountUnseenCountMsg struct {
	accountIndex int
	count        int
	err          error
}

// AccountSelectionModel handles the account selection screen for multi-account setups.
type AccountSelectionModel struct {
	cfg          *config.VeroConfig
	cursor       int
	choices      []string
	unseenCounts map[int]int
	loading      bool
}

// NewAccountSelectionModel creates a new account selection model.
func NewAccountSelectionModel(cfg *config.VeroConfig) AccountSelectionModel {
	choices := make([]string, len(cfg.Accounts))
	for i, account := range cfg.Accounts {
		choices[i] = account.Email
	}

	return AccountSelectionModel{
		cfg:          cfg,
		cursor:       0,
		choices:      choices,
		unseenCounts: make(map[int]int),
		loading:      true,
	}
}

func (m AccountSelectionModel) Init() tea.Cmd {
	cmds := make([]tea.Cmd, len(m.cfg.Accounts))
	for i := range m.cfg.Accounts {
		cmds[i] = m.fetchUnseenCountCmd(i)
	}
	return tea.Batch(cmds...)
}

func (m AccountSelectionModel) fetchUnseenCountCmd(accountIndex int) tea.Cmd {
	return func() tea.Msg {
		account := &m.cfg.Accounts[accountIndex]
		emails, err := email.FetchEmails(&account.IMAP, models.FilterUnseen)
		if err != nil {
			return accountUnseenCountMsg{accountIndex: accountIndex, count: 0, err: err}
		}
		return accountUnseenCountMsg{accountIndex: accountIndex, count: len(emails), err: nil}
	}
}

func (m AccountSelectionModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case accountUnseenCountMsg:
		if msg.err == nil {
			m.unseenCounts[msg.accountIndex] = msg.count
		}
		if len(m.unseenCounts) == len(m.cfg.Accounts) {
			m.loading = false
		}
		return m, nil

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			return m, tea.Quit

		case "up", "k":
			if m.cursor > 0 {
				m.cursor--
			}

		case "down", "j":
			if m.cursor < len(m.choices)-1 {
				m.cursor++
			}

		case "enter":
			selectedAccount := &m.cfg.Accounts[m.cursor]
			menu := NewMenuModel(m.cfg, selectedAccount)
			return menu, menu.Init()
		}
	}

	return m, nil
}

func (m AccountSelectionModel) View() string {
	s := titleStyle.Render("Vero - Select Account") + "\n\n"

	for i, choice := range m.choices {
		cursor := " "
		displayChoice := choice

		if count, ok := m.unseenCounts[i]; ok && count > 0 {
			displayChoice = fmt.Sprintf("%s (%d)", choice, count)
		}

		if m.cursor == i {
			cursor = ">"
			displayChoice = selectedStyle.Render(displayChoice)
		} else {
			displayChoice = normalStyle.Render(displayChoice)
		}
		s += fmt.Sprintf("  %s %s\n", cursor, displayChoice)
	}

	s += "\n" + helpStyle.Render("↑/↓ or j/k: navigate • enter: select • q: quit")

	return s
}
