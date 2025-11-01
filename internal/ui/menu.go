package ui

import (
	"fmt"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/models"
)

// MenuModel represents the main menu view where users select sections.
type MenuModel struct {
	account  *config.Account
	choices  []string
	cursor   int
	selected models.Section
}

// NewMenuModel creates a new main menu model for the specified account.
func NewMenuModel(account *config.Account) MenuModel {
	return MenuModel{
		account: account,
		choices: []string{"Inbox", "Sent", "Write"},
		cursor:  0,
	}
}

func (m MenuModel) Init() tea.Cmd {
	return nil
}

func (m MenuModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
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
			switch m.cursor {
			case 0:
				inbox := NewInboxModel(m.account)
				return inbox, inbox.Init()
			case 1:
				sent := NewSentModel(m.account)
				return sent, sent.Init()
			case 2:
				compose := NewComposeModel(m.account)
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
