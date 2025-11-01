package ui

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/viewport"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/muesli/reflow/wordwrap"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/models"
	"github.com/saravenpi/vero/internal/storage"
)

type sentLoadedMsg struct {
	emails []models.Email
	err    error
}

type SentModel struct {
	account       *config.Account
	emails        []models.Email
	cursor        int
	viewMode      models.ViewMode
	loading       bool
	err           error
	selectedIdx   int
	spinner       spinner.Model
	viewport      viewport.Model
	viewportReady bool
	windowWidth   int
	windowHeight  int
}

func NewSentModel(account *config.Account) SentModel {
	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = statusStyle

	vp := viewport.New(80, 20)
	vp.HighPerformanceRendering = false

	return SentModel{
		account:       account,
		viewMode:      models.ViewList,
		loading:       true,
		spinner:       s,
		viewport:      vp,
		viewportReady: true,
		windowWidth:   80,
		windowHeight:  30,
	}
}

func (m SentModel) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, m.loadSentEmailsCmd())
}

func (m SentModel) loadSentEmailsCmd() tea.Cmd {
	return func() tea.Msg {
		emails, err := storage.LoadSentEmails(m.account.Email)
		return sentLoadedMsg{emails: emails, err: err}
	}
}

func (m SentModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.windowWidth = msg.Width
		m.windowHeight = msg.Height
		m.viewport.Width = msg.Width - 4
		m.viewport.Height = msg.Height - 10
		if m.viewMode == models.ViewDetail && m.selectedIdx >= 0 && m.selectedIdx < len(m.emails) {
			m.updateViewportContent()
		}
		return m, nil

	case sentLoadedMsg:
		m.loading = false
		if msg.err != nil {
			m.err = msg.err
			return m, nil
		}
		m.emails = msg.emails
		return m, nil

	case spinner.TickMsg:
		// Only update spinner if we're actually loading
		if m.loading || (m.viewMode == models.ViewDetail && !m.viewportReady) {
			var cmd tea.Cmd
			m.spinner, cmd = m.spinner.Update(msg)
			return m, cmd
		}
		return m, nil

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			return m, tea.Quit

		case "esc":
			if m.viewMode == models.ViewDetail {
				m.viewMode = models.ViewList
				return m, nil
			}
			return NewMenuModel(m.account), nil

		case "up", "k":
			if m.viewMode == models.ViewList && m.cursor > 0 {
				m.cursor--
			} else if m.viewMode == models.ViewDetail {
				m.viewport.LineUp(1)
			}

		case "down", "j":
			if m.viewMode == models.ViewList && m.cursor < len(m.emails)-1 {
				m.cursor++
			} else if m.viewMode == models.ViewDetail {
				m.viewport.LineDown(1)
			}

		case "enter":
			if m.viewMode == models.ViewList && len(m.emails) > 0 {
				m.selectedIdx = m.cursor
				m.viewMode = models.ViewDetail
				m.viewport.GotoTop()
				m.updateViewportContent()
				return m, nil
			}

		default:
			if m.viewMode == models.ViewDetail {
				var cmd tea.Cmd
				m.viewport, cmd = m.viewport.Update(msg)
				return m, cmd
			}
		}
	}

	return m, nil
}

func (m SentModel) View() string {
	if m.loading {
		return fmt.Sprintf("\n  %s Loading sent emails...\n", m.spinner.View())
	}

	if m.err != nil {
		s := titleStyle.Render("Sent") + "\n\n"
		s += errorStyle.Render(fmt.Sprintf("Error: %v", m.err)) + "\n\n"
		s += helpStyle.Render("esc: back to menu")
		return s
	}

	if m.viewMode == models.ViewDetail {
		return m.renderDetail()
	}

	return m.renderList()
}

func (m SentModel) renderList() string {
	s := titleStyle.Render("Sent Emails") + "\n\n"

	if len(m.emails) == 0 {
		s += normalStyle.Render("  No sent emails found.") + "\n"
	} else {
		for i, em := range m.emails {
			cursor := " "
			line := fmt.Sprintf("To: %s - %s", em.To, em.Subject)

			if m.cursor == i {
				cursor = ">"
				line = selectedStyle.Render(line)
			} else {
				line = normalStyle.Render(line)
			}

			s += fmt.Sprintf("  %s %s\n", cursor, line)
		}
	}

	s += "\n" + helpStyle.Render("↑/↓ or j/k: navigate • enter: view • esc: back • q: quit")

	return s
}

func (m *SentModel) updateViewportContent() {
	if !m.viewportReady || m.selectedIdx < 0 || m.selectedIdx >= len(m.emails) {
		return
	}

	email := m.emails[m.selectedIdx]
	var content strings.Builder

	wrapWidth := m.viewport.Width
	if wrapWidth <= 0 {
		wrapWidth = 80
	}

	if email.Body == "" {
		content.WriteString(normalStyle.Render("(Empty body)") + "\n")
	} else {
		bodyLines := strings.Split(email.Body, "\n")
		for _, line := range bodyLines {
			if strings.TrimSpace(line) == "" {
				content.WriteString("\n")
				continue
			}

			wrappedLine := wordwrap.String(line, wrapWidth)
			wrappedLines := strings.Split(wrappedLine, "\n")
			for _, wl := range wrappedLines {
				if strings.HasPrefix(strings.TrimSpace(line), ">") {
					content.WriteString(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("  "+wl) + "\n")
				} else {
					content.WriteString(emailBodyStyle.Render(wl) + "\n")
				}
			}
		}
	}

	m.viewport.SetContent(content.String())
}

func (m SentModel) renderDetail() string {
	email := m.emails[m.selectedIdx]

	s := titleStyle.Render("Sent Email Details") + "\n\n"

	s += emailHeaderStyle.Render("  To: ") + normalStyle.Render(email.To) + "\n"
	if email.CC != "" {
		s += emailHeaderStyle.Render("  CC: ") + normalStyle.Render(email.CC) + "\n"
	}
	s += emailHeaderStyle.Render("  Subject: ") + normalStyle.Render(email.Subject) + "\n"
	s += emailHeaderStyle.Render("  Date: ") + normalStyle.Render(email.Date) + "\n\n"

	if m.viewportReady {
		s += m.viewport.View()
	} else {
		s += fmt.Sprintf("  %s Preparing view...\n", m.spinner.View())
	}

	scrollInfo := ""
	if m.viewportReady {
		scrollPercent := int(m.viewport.ScrollPercent() * 100)
		scrollInfo = fmt.Sprintf(" • %d%%", scrollPercent)
	}

	s += "\n" + helpStyle.Render("↑↓/j/k: scroll • esc: back to list • q: quit"+scrollInfo)

	return s
}
