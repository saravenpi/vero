package ui

import (
	"fmt"
	"os"
	"path/filepath"
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
	stepAttachments
	stepPreview
	stepSending
	stepDone
)

type emailSentMsg struct {
	err error
}

// ComposeModel manages the email composition workflow through multiple steps.
type ComposeModel struct {
	account            *config.Account
	step               composeStep
	toInput            textinput.Model
	ccInput            textinput.Model
	subjInput          textinput.Model
	bodyInput          textarea.Model
	attachInput        textinput.Model
	draft              models.EmailDraft
	selectedAttachIdx  int
	err                error
	success            bool
	spinner            spinner.Model
	completions        []string
	completionIndex    int
	lastCompletionPath string
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

	attach := textinput.New()
	attach.Placeholder = "/path/to/file.pdf"
	attach.CharLimit = 512
	attach.Width = 60

	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = statusStyle

	return ComposeModel{
		account:            account,
		step:               stepTo,
		toInput:            ti,
		ccInput:            cc,
		subjInput:          subj,
		bodyInput:          ta,
		attachInput:        attach,
		selectedAttachIdx:  0,
		spinner:            s,
		completions:        []string{},
		completionIndex:    -1,
		lastCompletionPath: "",
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
				m.step = stepAttachments
				m.attachInput.Focus()
				return m, nil
			}
			if m.step == stepAttachments {
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

			case stepAttachments:
				filePath := m.attachInput.Value()
				if filePath != "" {
					if err := m.addAttachment(filePath); err != nil {
						m.err = err
					} else {
						m.attachInput.SetValue("")
						m.completions = []string{}
						m.completionIndex = -1
						m.lastCompletionPath = ""
					}
				} else {
					m.step = stepPreview
					m.attachInput.Blur()
				}
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
				m.step = stepAttachments
				m.bodyInput.Blur()
				m.attachInput.Focus()
				return m, nil
			}
		case "ctrl+n":
			if m.step == stepAttachments {
				m.step = stepPreview
				m.attachInput.Blur()
				return m, nil
			}
		case "backspace":
			if m.step == stepAttachments && m.attachInput.Value() == "" && len(m.draft.Attachments) > 0 {
				if m.selectedAttachIdx >= 0 && m.selectedAttachIdx < len(m.draft.Attachments) {
					m.draft.Attachments = append(
						m.draft.Attachments[:m.selectedAttachIdx],
						m.draft.Attachments[m.selectedAttachIdx+1:]...,
					)
					if m.selectedAttachIdx >= len(m.draft.Attachments) && len(m.draft.Attachments) > 0 {
						m.selectedAttachIdx = len(m.draft.Attachments) - 1
					}
				}
				return m, nil
			}
		case "up":
			if m.step == stepAttachments && m.attachInput.Value() == "" && len(m.draft.Attachments) > 0 {
				if m.selectedAttachIdx > 0 {
					m.selectedAttachIdx--
				}
				return m, nil
			}
		case "down":
			if m.step == stepAttachments && m.attachInput.Value() == "" && len(m.draft.Attachments) > 0 {
				if m.selectedAttachIdx < len(m.draft.Attachments)-1 {
					m.selectedAttachIdx++
				}
				return m, nil
			}
		case "tab":
			if m.step == stepAttachments {
				currentPath := m.attachInput.Value()
				if currentPath == "" {
					return m, nil
				}

				completion, completions, index := getNextCompletion(currentPath, m.completions, m.completionIndex)

				if completion != currentPath {
					m.attachInput.SetValue(completion)
					m.completions = completions
					m.completionIndex = index
					m.lastCompletionPath = currentPath
				} else if len(completions) > 1 {
					commonPrefixPath := getCommonPrefix(currentPath)
					if commonPrefixPath != currentPath {
						m.attachInput.SetValue(commonPrefixPath)
						m.completions = []string{}
						m.completionIndex = -1
					}
				}
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
	case stepAttachments:
		oldValue := m.attachInput.Value()
		m.attachInput, cmd = m.attachInput.Update(msg)
		newValue := m.attachInput.Value()

		if oldValue != newValue && newValue != m.lastCompletionPath {
			m.completions = []string{}
			m.completionIndex = -1
		}
	}

	return m, cmd
}

func (m *ComposeModel) addAttachment(filePath string) error {
	filePath = strings.TrimSpace(filePath)

	if strings.HasPrefix(filePath, "~/") {
		home, err := os.UserHomeDir()
		if err == nil {
			filePath = filepath.Join(home, filePath[2:])
		}
	}

	info, err := os.Stat(filePath)
	if err != nil {
		return fmt.Errorf("file not found: %s", filePath)
	}

	if info.IsDir() {
		return fmt.Errorf("cannot attach directory: %s", filePath)
	}

	filename := filepath.Base(filePath)

	for _, att := range m.draft.Attachments {
		if att.FilePath == filePath {
			return fmt.Errorf("file already attached: %s", filename)
		}
	}

	attachment := models.Attachment{
		Filename:    filename,
		ContentType: "",
		Size:        info.Size(),
		FilePath:    filePath,
	}

	m.draft.Attachments = append(m.draft.Attachments, attachment)
	m.selectedAttachIdx = len(m.draft.Attachments) - 1

	return nil
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
		s += helpStyle.Render("\n  ctrl+d: attachments • esc: cancel")

	case stepAttachments:
		s += labelStyle.Render("  To: ") + normalStyle.Render(m.draft.To) + "\n"
		if m.draft.CC != "" {
			s += labelStyle.Render("  CC: ") + normalStyle.Render(m.draft.CC) + "\n"
		}
		s += labelStyle.Render("  Subject: ") + normalStyle.Render(m.draft.Subject) + "\n"

		if len(m.draft.Attachments) > 0 {
			s += "\n" + attachmentHeaderStyle.Render("Attachments:") + "\n"
			s += renderAttachmentList(m.draft.Attachments, m.selectedAttachIdx, m.attachInput.Value() == "")
		}

		s += "\n" + labelStyle.Render("  Add file: ") + m.attachInput.View() + "\n"

		if m.err != nil {
			s += "\n" + errorStyle.Render(fmt.Sprintf("  Error: %v", m.err)) + "\n"
			m.err = nil
		}

		helpText := "tab: autocomplete • enter: add file"
		if len(m.draft.Attachments) > 0 {
			helpText += " • up/down: select • backspace: remove"
		}
		helpText += " • ctrl+n: next • esc: back"
		s += helpStyle.Render("\n  " + helpText)

	case stepPreview:
		s += emailHeaderStyle.Render("  To: ") + normalStyle.Render(m.draft.To) + "\n"
		if m.draft.CC != "" {
			s += emailHeaderStyle.Render("  CC: ") + normalStyle.Render(m.draft.CC) + "\n"
		}
		s += emailHeaderStyle.Render("  Subject: ") + normalStyle.Render(m.draft.Subject) + "\n"

		if len(m.draft.Attachments) > 0 {
			s += emailHeaderStyle.Render("  Attachments: ") + normalStyle.Render(fmt.Sprintf("%d file(s)", len(m.draft.Attachments))) + "\n"
		}

		s += "\n"

		bodyLines := strings.Split(m.draft.Body, "\n")
		for _, line := range bodyLines {
			s += emailBodyStyle.Render(line) + "\n"
		}

		if len(m.draft.Attachments) > 0 {
			s += "\n" + attachmentHeaderStyle.Render("Attached files:") + "\n"
			s += renderAttachmentList(m.draft.Attachments, -1, false)
		}

		s += "\n" + helpStyle.Render("enter: send • esc: edit attachments")
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
