package ui

import (
	"fmt"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/saravenpi/vero/internal/config"
)

// AccountSelectionModel handles the account selection screen for multi-account setups.
type AccountSelectionModel struct {
	cfg     *config.VeroConfig
	cursor  int
	choices []string
}

// NewAccountSelectionModel creates a new account selection model.
func NewAccountSelectionModel(cfg *config.VeroConfig) AccountSelectionModel {
	choices := make([]string, len(cfg.Accounts))
	for i, account := range cfg.Accounts {
		choices[i] = account.Email
	}

	return AccountSelectionModel{
		cfg:     cfg,
		cursor:  0,
		choices: choices,
	}
}

func (m AccountSelectionModel) Init() tea.Cmd {
	return nil
}

func (m AccountSelectionModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
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
			menu := NewMenuModel(selectedAccount)
			return menu, menu.Init()
		}
	}

	return m, nil
}

func (m AccountSelectionModel) View() string {
	s := titleStyle.Render("Vero - Select Account") + "\n\n"

	for i, choice := range m.choices {
		cursor := " "
		if m.cursor == i {
			cursor = ">"
			choice = selectedStyle.Render(choice)
		} else {
			choice = normalStyle.Render(choice)
		}
		s += fmt.Sprintf("  %s %s\n", cursor, choice)
	}

	s += "\n" + helpStyle.Render("↑/↓ or j/k: navigate • enter: select • q: quit")

	return s
}
