package ui

import (
	"fmt"
	"os"
	"os/exec"
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

type editorFinishedMsg struct {
	body string
	err  error
}

// ComposeModel manages the email composition workflow through multiple steps.
type ComposeModel struct {
	cfg                *config.VeroConfig
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
	usingExternalEditor bool
}

// NewComposeModel creates a new email composition model for the specified account.
func NewComposeModel(cfg *config.VeroConfig, account *config.Account) ComposeModel {
	ti := textinput.New()
	ti.Placeholder = "recipient@example.com"
	ti.Focus()
	ti.CharLimit = 256
	ti.Width = 0

	cc := textinput.New()
	cc.Placeholder = "cc@example.com (optional)"
	cc.CharLimit = 256
	cc.Width = 0

	subj := textinput.New()
	subj.Placeholder = "Email subject"
	subj.CharLimit = 256
	subj.Width = 0

	ta := textarea.New()
	ta.Placeholder = "Type your message here..."
	ta.SetWidth(0)
	ta.SetHeight(0)

	attach := textinput.New()
	attach.Placeholder = "/path/to/file.pdf"
	attach.CharLimit = 512
	attach.Width = 0

	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = statusStyle

	return ComposeModel{
		cfg:                cfg,
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
	case tea.WindowSizeMsg:
		m.toInput.Width = msg.Width - 10
		m.ccInput.Width = msg.Width - 10
		m.subjInput.Width = msg.Width - 10
		m.bodyInput.SetWidth(msg.Width - 4)
		m.bodyInput.SetHeight(msg.Height - 12)
		m.attachInput.Width = msg.Width - 15
		return m, nil

	case emailSentMsg:
		m.step = stepDone
		if msg.err != nil {
			m.err = msg.err
			m.success = false
		} else {
			m.success = true
		}
		return m, nil

	case editorFinishedMsg:
		m.usingExternalEditor = false
		if msg.err != nil {
			m.err = msg.err
			m.step = stepSubject
			m.subjInput.Focus()
			return m, nil
		}
		m.draft.Body = msg.body
		m.bodyInput.SetValue(msg.body)
		m.step = stepAttachments
		m.attachInput.Focus()
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
				if m.cfg.Editor != "" {
					m.step = stepBody
					m.usingExternalEditor = true
					return m, m.openExternalEditorCmd()
				}
				m.step = stepBody
				m.bodyInput.Focus()
				return m, nil
			}
			if m.step == stepDone {
				return NewMenuModel(m.cfg, m.account), nil
			}
			return NewMenuModel(m.cfg, m.account), nil

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

				if m.cfg.Editor != "" {
					m.usingExternalEditor = true
					return m, m.openExternalEditorCmd()
				}

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
				return NewMenuModel(m.cfg, m.account), nil
			}

		case "ctrl+d":
			if m.step == stepBody && !m.usingExternalEditor {
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
		if !m.usingExternalEditor {
			m.bodyInput, cmd = m.bodyInput.Update(msg)
		}
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
		if m.err != nil {
			s += "\n" + errorStyle.Render(fmt.Sprintf("  Error: %v", m.err)) + "\n"
			s += helpStyle.Render("\n  enter: retry • esc: cancel")
			m.err = nil
		} else {
			s += helpStyle.Render("\n  enter: next • esc: cancel")
		}

	case stepBody:
		s += labelStyle.Render("  To: ") + normalStyle.Render(m.draft.To) + "\n"
		if m.draft.CC != "" {
			s += labelStyle.Render("  CC: ") + normalStyle.Render(m.draft.CC) + "\n"
		}
		s += labelStyle.Render("  Subject: ") + normalStyle.Render(m.draft.Subject) + "\n\n"

		if m.usingExternalEditor {
			s += statusStyle.Render("  Opening external editor...") + "\n"
			if m.err != nil {
				s += "\n" + errorStyle.Render(fmt.Sprintf("  Error: %v", m.err)) + "\n"
				m.err = nil
			}
		} else {
			s += labelStyle.Render("  Body:\n") + m.bodyInput.View() + "\n"
			s += helpStyle.Render("\n  ctrl+d: attachments • esc: cancel")
		}

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

func (m ComposeModel) openExternalEditorCmd() tea.Cmd {
	tmpfile, err := os.CreateTemp("", "vero-email-*.txt")
	if err != nil {
		return func() tea.Msg {
			return editorFinishedMsg{err: fmt.Errorf("failed to create temp file: %w", err)}
		}
	}
	tmpPath := tmpfile.Name()

	if m.draft.Body != "" {
		if _, err := tmpfile.WriteString(m.draft.Body); err != nil {
			tmpfile.Close()
			os.Remove(tmpPath)
			return func() tea.Msg {
				return editorFinishedMsg{err: fmt.Errorf("failed to write to temp file: %w", err)}
			}
		}
	}
	tmpfile.Close()

	c := exec.Command(m.cfg.Editor, tmpPath)
	return tea.ExecProcess(c, func(err error) tea.Msg {
		defer os.Remove(tmpPath)

		if err != nil {
			return editorFinishedMsg{err: fmt.Errorf("editor exited with error: %w", err)}
		}

		content, err := os.ReadFile(tmpPath)
		if err != nil {
			return editorFinishedMsg{err: fmt.Errorf("failed to read temp file: %w", err)}
		}

		return editorFinishedMsg{body: string(content), err: nil}
	})
}
