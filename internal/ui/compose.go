package ui

import (
	"fmt"
	"strings"
	"time"

	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/textarea"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/email"
	"github.com/saravenpi/vero/internal/models"
	"github.com/saravenpi/vero/internal/storage"
)

type composeStep int

const (
	stepTo composeStep = iota
	stepCC
	stepSubject
	stepBody
	stepPreview
	stepSending
	stepDone
)

type emailSentMsg struct {
	err error
}

// ComposeModel manages the email composition workflow through multiple steps.
type ComposeModel struct {
	account   *config.Account
	step      composeStep
	toInput   textinput.Model
	ccInput   textinput.Model
	subjInput textinput.Model
	bodyInput textarea.Model
	draft     models.EmailDraft
	err       error
	success   bool
	spinner   spinner.Model
}

// NewComposeModel creates a new email composition model for the specified account.
func NewComposeModel(account *config.Account) ComposeModel {
	ti := textinput.New()
	ti.Placeholder = "recipient@example.com"
	ti.Focus()
	ti.CharLimit = 256
	ti.Width = 50

	cc := textinput.New()
	cc.Placeholder = "cc@example.com (optional)"
	cc.CharLimit = 256
	cc.Width = 50

	subj := textinput.New()
	subj.Placeholder = "Email subject"
	subj.CharLimit = 256
	subj.Width = 50

	ta := textarea.New()
	ta.Placeholder = "Type your message here..."
	ta.SetWidth(80)
	ta.SetHeight(10)

	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = statusStyle

	return ComposeModel{
		account:   account,
		step:      stepTo,
		toInput:   ti,
		ccInput:   cc,
		subjInput: subj,
		bodyInput: ta,
		spinner:   s,
	}
}

func (m ComposeModel) Init() tea.Cmd {
	return tea.Batch(textinput.Blink, m.spinner.Tick)
}

func (m ComposeModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmd tea.Cmd

	switch msg := msg.(type) {
	case emailSentMsg:
		m.step = stepDone
		if msg.err != nil {
			m.err = msg.err
			m.success = false
		} else {
			m.success = true
		}
		return m, nil

	case spinner.TickMsg:
		if m.step == stepSending {
			var spinnerCmd tea.Cmd
			m.spinner, spinnerCmd = m.spinner.Update(msg)
			return m, spinnerCmd
		}
		return m, nil

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c":
			return m, tea.Quit

		case "esc":
			if m.step == stepPreview {
				m.step = stepBody
				m.bodyInput.Focus()
				return m, nil
			}
			if m.step == stepDone {
				return NewMenuModel(m.account), nil
			}
			return NewMenuModel(m.account), nil

		case "enter":
			switch m.step {
			case stepTo:
				m.draft.To = m.toInput.Value()
				if m.draft.To == "" {
					return m, nil
				}
				m.step = stepCC
				m.toInput.Blur()
				m.ccInput.Focus()
				return m, nil

			case stepCC:
				m.draft.CC = m.ccInput.Value()
				m.step = stepSubject
				m.ccInput.Blur()
				m.subjInput.Focus()
				return m, nil

			case stepSubject:
				m.draft.Subject = m.subjInput.Value()
				if m.draft.Subject == "" {
					return m, nil
				}
				m.step = stepBody
				m.subjInput.Blur()
				m.bodyInput.Focus()
				return m, nil

			case stepPreview:
				m.step = stepSending
				return m, tea.Batch(m.spinner.Tick, m.sendEmailCmd())

			case stepDone:
				return NewMenuModel(m.account), nil
			}

		case "ctrl+d":
			if m.step == stepBody {
				m.draft.Body = m.bodyInput.Value()
				if m.draft.Body == "" {
					return m, nil
				}
				m.step = stepPreview
				m.bodyInput.Blur()
				return m, nil
			}
		}
	}

	switch m.step {
	case stepTo:
		m.toInput, cmd = m.toInput.Update(msg)
	case stepCC:
		m.ccInput, cmd = m.ccInput.Update(msg)
	case stepSubject:
		m.subjInput, cmd = m.subjInput.Update(msg)
	case stepBody:
		m.bodyInput, cmd = m.bodyInput.Update(msg)
	}

	return m, cmd
}

func (m ComposeModel) View() string {
	if m.step == stepSending {
		return fmt.Sprintf("\n  %s Sending email...\n", m.spinner.View())
	}

	if m.step == stepDone {
		if m.success {
			s := titleStyle.Render("Email Sent!") + "\n\n"
			s += normalStyle.Render("  Your email has been sent successfully.") + "\n\n"
			s += helpStyle.Render("enter or esc: back to menu")
			return s
		} else {
			s := titleStyle.Render("Send Failed") + "\n\n"
			s += errorStyle.Render(fmt.Sprintf("  Error: %v", m.err)) + "\n\n"
			s += helpStyle.Render("enter or esc: back to menu")
			return s
		}
	}

	s := titleStyle.Render("Compose Email") + "\n\n"

	switch m.step {
	case stepTo:
		s += labelStyle.Render("  To: ") + m.toInput.View() + "\n"
		s += helpStyle.Render("\n  enter: next • esc: cancel")

	case stepCC:
		s += labelStyle.Render("  To: ") + normalStyle.Render(m.draft.To) + "\n"
		s += labelStyle.Render("  CC: ") + m.ccInput.View() + "\n"
		s += helpStyle.Render("\n  enter: next (optional) • esc: cancel")

	case stepSubject:
		s += labelStyle.Render("  To: ") + normalStyle.Render(m.draft.To) + "\n"
		if m.draft.CC != "" {
			s += labelStyle.Render("  CC: ") + normalStyle.Render(m.draft.CC) + "\n"
		}
		s += labelStyle.Render("  Subject: ") + m.subjInput.View() + "\n"
		s += helpStyle.Render("\n  enter: next • esc: cancel")

	case stepBody:
		s += labelStyle.Render("  To: ") + normalStyle.Render(m.draft.To) + "\n"
		if m.draft.CC != "" {
			s += labelStyle.Render("  CC: ") + normalStyle.Render(m.draft.CC) + "\n"
		}
		s += labelStyle.Render("  Subject: ") + normalStyle.Render(m.draft.Subject) + "\n\n"
		s += labelStyle.Render("  Body:\n") + m.bodyInput.View() + "\n"
		s += helpStyle.Render("\n  ctrl+d: preview • esc: cancel")

	case stepPreview:
		s += emailHeaderStyle.Render("  To: ") + normalStyle.Render(m.draft.To) + "\n"
		if m.draft.CC != "" {
			s += emailHeaderStyle.Render("  CC: ") + normalStyle.Render(m.draft.CC) + "\n"
		}
		s += emailHeaderStyle.Render("  Subject: ") + normalStyle.Render(m.draft.Subject) + "\n\n"

		bodyLines := strings.Split(m.draft.Body, "\n")
		for _, line := range bodyLines {
			s += emailBodyStyle.Render(line) + "\n"
		}

		s += "\n" + helpStyle.Render("enter: send • esc: edit body")
	}

	return s
}

func (m ComposeModel) sendEmailCmd() tea.Cmd {
	return func() tea.Msg {
		err := email.SendEmail(&m.account.SMTP, m.draft)
		if err != nil {
			return emailSentMsg{err: err}
		}

		sentEmail := models.Email{
			To:      m.draft.To,
			CC:      m.draft.CC,
			Subject: m.draft.Subject,
			Body:    m.draft.Body,
			Date:    time.Now().Format(time.RFC1123Z),
		}
		storage.SaveSentEmail(m.account.Email, sentEmail)

		return emailSentMsg{err: nil}
	}
}
