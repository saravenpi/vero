package ui

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/charmbracelet/bubbles/list"
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/viewport"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/muesli/reflow/wordwrap"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/email"
	"github.com/saravenpi/vero/internal/models"
	"github.com/saravenpi/vero/internal/storage"
)

type emailsFetchedMsg struct {
	emails []models.Email
	err    error
}

type emailBodyFetchedMsg struct {
	body        string
	attachments []models.Attachment
	err         error
}

type emailItem struct {
	email models.Email
	index int
}

func (i emailItem) FilterValue() string {
	return i.email.Subject + " " + i.email.From
}

func (i emailItem) Title() string {
	return i.email.Subject
}

func (i emailItem) Description() string {
	timeAgo := formatTimeAgo(i.email.Timestamp)
	return fmt.Sprintf("%s • %s", i.email.From, timeAgo)
}

func formatTimeAgo(t time.Time) string {
	if t.IsZero() {
		return "unknown"
	}

	now := time.Now()
	duration := now.Sub(t)

	if duration < time.Minute {
		return "just now"
	}
	if duration < 2*time.Minute {
		return "1 minute ago"
	}
	if duration < time.Hour {
		minutes := int(duration.Minutes())
		return fmt.Sprintf("%d minutes ago", minutes)
	}
	if duration < 2*time.Hour {
		return "1 hour ago"
	}
	if duration < 24*time.Hour {
		hours := int(duration.Hours())
		return fmt.Sprintf("%d hours ago", hours)
	}
	if duration < 48*time.Hour {
		return "1 day ago"
	}
	if duration < 7*24*time.Hour {
		days := int(duration.Hours() / 24)
		return fmt.Sprintf("%d days ago", days)
	}
	if duration < 14*24*time.Hour {
		return "1 week ago"
	}
	if duration < 30*24*time.Hour {
		weeks := int(duration.Hours() / 24 / 7)
		return fmt.Sprintf("%d weeks ago", weeks)
	}
	if duration < 60*24*time.Hour {
		return "1 month ago"
	}
	if duration < 365*24*time.Hour {
		months := int(duration.Hours() / 24 / 30)
		return fmt.Sprintf("%d months ago", months)
	}
	if duration < 2*365*24*time.Hour {
		return "1 year ago"
	}
	years := int(duration.Hours() / 24 / 365)
	return fmt.Sprintf("%d years ago", years)
}

// InboxModel manages the inbox view with email list and detail views.
type InboxModel struct {
	cfg               *config.VeroConfig
	account           *config.Account
	emails            []models.Email
	list              list.Model
	viewMode          models.ViewMode
	filter            models.InboxFilter
	loading           bool
	err               error
	selectedIdx       int
	selectedAttachIdx int
	spinner           spinner.Model
	loadingBody       bool
	viewport          viewport.Model
	viewportReady     bool
	windowWidth       int
	windowHeight      int
}

// NewInboxModel creates a new inbox model for the specified account.
func NewInboxModel(cfg *config.VeroConfig, account *config.Account) InboxModel {
	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = statusStyle

	delegate := list.NewDefaultDelegate()
	delegate.Styles.SelectedTitle = delegate.Styles.SelectedTitle.
		Foreground(lipgloss.Color("5")).
		Bold(true)
	delegate.Styles.SelectedDesc = delegate.Styles.SelectedDesc.
		Foreground(lipgloss.Color("8"))

	l := list.New([]list.Item{}, delegate, 80, 20)
	l.Title = "Inbox"
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false)
	l.SetShowHelp(false)

	vp := viewport.New(80, 20)
	vp.HighPerformanceRendering = false

	defaultFilter := models.FilterAll
	switch cfg.DefaultInboxView {
	case "unseen":
		defaultFilter = models.FilterUnseen
	case "seen":
		defaultFilter = models.FilterSeen
	case "all":
		defaultFilter = models.FilterAll
	}

	m := InboxModel{
		cfg:           cfg,
		account:       account,
		list:          l,
		viewMode:      models.ViewList,
		filter:        defaultFilter,
		loading:       true,
		spinner:       s,
		viewport:      vp,
		viewportReady: true,
		windowWidth:   80,
		windowHeight:  30,
	}
	return m
}

func (m InboxModel) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, m.fetchEmailsCmd())
}

func (m InboxModel) fetchEmailsCmd() tea.Cmd {
	return func() tea.Msg {
		emails, err := email.FetchEmails(&m.account.IMAP, m.filter)
		if err != nil {
			return emailsFetchedMsg{emails: nil, err: err}
		}
		return emailsFetchedMsg{emails: emails, err: nil}
	}
}

func (m InboxModel) fetchBodyCmd(uid uint32) tea.Cmd {
	return func() tea.Msg {
		body, attachments, err := email.FetchEmailBodyAndAttachments(&m.account.IMAP, uid)
		if err != nil {
			return emailBodyFetchedMsg{body: "", attachments: nil, err: err}
		}
		return emailBodyFetchedMsg{body: body, attachments: attachments, err: nil}
	}
}

func (m InboxModel) openAttachmentCmd(attachment models.Attachment) tea.Cmd {
	return func() tea.Msg {
		cmd := exec.Command("open", attachment.FilePath)
		_ = cmd.Start()
		return nil
	}
}

func (m InboxModel) downloadAttachmentCmd(attachment models.Attachment) tea.Cmd {
	return func() tea.Msg {
		downloadsDir := m.cfg.DownloadFolder
		destPath := filepath.Join(downloadsDir, attachment.Filename)

		counter := 1
		originalPath := destPath
		for {
			if _, err := os.Stat(destPath); os.IsNotExist(err) {
				break
			}
			ext := filepath.Ext(originalPath)
			nameWithoutExt := strings.TrimSuffix(filepath.Base(originalPath), ext)
			destPath = filepath.Join(downloadsDir, fmt.Sprintf("%s (%d)%s", nameWithoutExt, counter, ext))
			counter++
		}

		data, err := os.ReadFile(attachment.FilePath)
		if err != nil {
			return nil
		}

		err = os.WriteFile(destPath, data, 0644)
		if err != nil {
			return nil
		}

		return nil
	}
}

func (m InboxModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
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

	case emailsFetchedMsg:
		m.loading = false
		if msg.err != nil {
			m.err = msg.err
			return m, nil
		}
		m.emails = msg.emails

		items := make([]list.Item, len(msg.emails))
		for i, em := range msg.emails {
			items[i] = emailItem{email: em, index: i}
		}
		m.list.SetItems(items)
		m.list.Title = fmt.Sprintf("Inbox (%s) - %d emails", m.filter.String(), len(items))
		return m, nil

	case emailBodyFetchedMsg:
		m.loadingBody = false
		if msg.err != nil {
			m.err = msg.err
			return m, nil
		}
		if m.selectedIdx >= 0 && m.selectedIdx < len(m.emails) {
			m.emails[m.selectedIdx].Body = msg.body
			m.emails[m.selectedIdx].Attachments = msg.attachments
			m.updateViewportContent()
			go storage.SaveSeenEmail(m.account.Email, m.emails[m.selectedIdx])
		}
		return m, nil

	case spinner.TickMsg:
		if m.loading || m.loadingBody || (m.viewMode == models.ViewDetail && !m.viewportReady) {
			var cmd tea.Cmd
			m.spinner, cmd = m.spinner.Update(msg)
			return m, cmd
		}
		return m, nil

	case tea.KeyMsg:
		if msg.String() == "ctrl+c" || msg.String() == "q" {
			return m, tea.Quit
		}

		if msg.String() == "esc" {
			if m.viewMode == models.ViewDetail {
				m.viewMode = models.ViewList
				m.err = nil
				return m, nil
			}
			menu := NewMenuModel(m.cfg, m.account)
			return menu, menu.Init()
		}

		if m.loading || m.loadingBody {
			return m, nil
		}

		switch msg.String() {
		case "u":
			if m.viewMode == models.ViewList {
				m.filter = models.FilterUnseen
				m.loading = true
				return m, tea.Batch(m.spinner.Tick, m.fetchEmailsCmd())
			}

		case "s":
			if m.viewMode == models.ViewList {
				m.filter = models.FilterSeen
				m.loading = true
				return m, tea.Batch(m.spinner.Tick, m.fetchEmailsCmd())
			}

		case "a":
			if m.viewMode == models.ViewList {
				m.filter = models.FilterAll
				m.loading = true
				return m, tea.Batch(m.spinner.Tick, m.fetchEmailsCmd())
			}

		case "r":
			if m.viewMode == models.ViewList {
				m.loading = true
				return m, tea.Batch(m.spinner.Tick, m.fetchEmailsCmd())
			}

		case "enter":
			if m.viewMode == models.ViewList && len(m.emails) > 0 {
				if item, ok := m.list.SelectedItem().(emailItem); ok {
					m.selectedIdx = item.index
					m.selectedAttachIdx = 0
					m.viewMode = models.ViewDetail
					m.viewport.GotoTop()

					selectedEmail := m.emails[item.index]
					if selectedEmail.Body == "" {
						m.loadingBody = true
						return m, tea.Batch(
							m.spinner.Tick,
							m.fetchBodyCmd(selectedEmail.UID),
						)
					}

					m.updateViewportContent()
					go storage.SaveSeenEmail(m.account.Email, m.emails[item.index])
					return m, nil
				}
			}

		case "o":
			if m.viewMode == models.ViewDetail && m.selectedIdx >= 0 && m.selectedIdx < len(m.emails) {
				email := m.emails[m.selectedIdx]
				if len(email.Attachments) > 0 && m.selectedAttachIdx >= 0 && m.selectedAttachIdx < len(email.Attachments) {
					attachment := email.Attachments[m.selectedAttachIdx]
					return m, m.openAttachmentCmd(attachment)
				}
			}

		case "d":
			if m.viewMode == models.ViewDetail && m.selectedIdx >= 0 && m.selectedIdx < len(m.emails) {
				email := m.emails[m.selectedIdx]
				if len(email.Attachments) > 0 && m.selectedAttachIdx >= 0 && m.selectedAttachIdx < len(email.Attachments) {
					attachment := email.Attachments[m.selectedAttachIdx]
					return m, m.downloadAttachmentCmd(attachment)
				}
			}

		case "left", "h":
			if m.viewMode == models.ViewDetail && m.selectedIdx >= 0 && m.selectedIdx < len(m.emails) {
				email := m.emails[m.selectedIdx]
				if len(email.Attachments) > 0 && m.selectedAttachIdx > 0 {
					m.selectedAttachIdx--
					return m, nil
				}
			}

		case "right", "l":
			if m.viewMode == models.ViewDetail && m.selectedIdx >= 0 && m.selectedIdx < len(m.emails) {
				email := m.emails[m.selectedIdx]
				if len(email.Attachments) > 0 && m.selectedAttachIdx < len(email.Attachments)-1 {
					m.selectedAttachIdx++
					return m, nil
				}
			}

		default:
			if m.viewMode == models.ViewList {
				var cmd tea.Cmd
				m.list, cmd = m.list.Update(msg)
				return m, cmd
			}
			if m.viewMode == models.ViewDetail {
				var cmd tea.Cmd
				m.viewport, cmd = m.viewport.Update(msg)
				return m, cmd
			}
		}
	}

	return m, nil
}

func (m InboxModel) View() string {
	if m.loading {
		return fmt.Sprintf("\n  %s Loading emails...\n", m.spinner.View())
	}

	if m.err != nil && m.viewMode != models.ViewDetail {
		s := titleStyle.Render("Inbox") + "\n\n"
		s += errorStyle.Render(fmt.Sprintf("Error: %v", m.err)) + "\n\n"
		s += helpStyle.Render("esc: back to menu")
		return s
	}

	if m.viewMode == models.ViewDetail {
		return m.renderDetail()
	}

	return m.renderList()
}

func (m InboxModel) renderList() string {
	if len(m.emails) == 0 {
		s := titleStyle.Render(fmt.Sprintf("Inbox (%s)", m.filter.String())) + "\n\n"
		s += normalStyle.Render("  No emails found.") + "\n"
		s += "\n" + helpStyle.Render("u/s/a: filter • r: refresh • esc: back • q: quit")
		return s
	}

	return m.list.View() + "\n" + helpStyle.Render("↑↓/jk: navigate • enter: read • u/s/a: filter • r: refresh • esc: back • q: quit")
}

func (m *InboxModel) updateViewportContent() {
	if !m.viewportReady || m.selectedIdx < 0 || m.selectedIdx >= len(m.emails) {
		return
	}

	email := m.emails[m.selectedIdx]
	var content strings.Builder

	wrapWidth := m.viewport.Width
	if wrapWidth <= 0 {
		wrapWidth = 80
	}

	if m.err != nil && email.Body == "" {
		content.WriteString(errorStyle.Render(fmt.Sprintf("Error loading email body: %v", m.err)) + "\n\n")
		content.WriteString(normalStyle.Render("This email may have an unsupported format or only contain attachments.") + "\n")
	} else if email.Body == "" {
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

func (m InboxModel) renderDetail() string {
	email := m.emails[m.selectedIdx]

	s := titleStyle.Render("Email Details") + "\n\n"

	s += emailHeaderStyle.Render("  From: ") + normalStyle.Render(email.From) + "\n"
	if email.To != "" {
		s += emailHeaderStyle.Render("  To: ") + normalStyle.Render(email.To) + "\n"
	}
	if email.CC != "" {
		s += emailHeaderStyle.Render("  CC: ") + normalStyle.Render(email.CC) + "\n"
	}
	s += emailHeaderStyle.Render("  Subject: ") + normalStyle.Render(email.Subject) + "\n"
	s += emailHeaderStyle.Render("  Date: ") + normalStyle.Render(email.Date) + "\n"

	if len(email.Attachments) > 0 {
		s += "\n" + attachmentHeaderStyle.Render("Attachments:") + "\n"
		for i, att := range email.Attachments {
			icon := getFileIcon(att.Filename)
			sizeStr := formatFileSize(att.Size)

			if i == m.selectedAttachIdx {
				s += attachmentSelectedStyle.Render(fmt.Sprintf("▶ %s %s (%s)", icon, att.Filename, sizeStr)) + "\n"
			} else {
				s += attachmentStyle.Render(fmt.Sprintf("  %s %s (%s)", icon, att.Filename, sizeStr)) + "\n"
			}
		}
	}

	s += "\n"

	if m.loadingBody {
		s += fmt.Sprintf("  %s Loading email content...\n", m.spinner.View())
	} else if m.viewportReady {
		s += m.viewport.View()
	} else {
		s += fmt.Sprintf("  %s Preparing view...\n", m.spinner.View())
	}

	scrollInfo := ""
	if m.viewportReady && !m.loadingBody {
		scrollPercent := int(m.viewport.ScrollPercent() * 100)
		scrollInfo = fmt.Sprintf(" • %d%%", scrollPercent)
	}

	helpText := "↑↓/j/k: scroll"
	if len(email.Attachments) > 0 {
		helpText += " • ←→/h/l: select • o: open • d: download"
	}
	helpText += " • esc: back • q: quit" + scrollInfo

	s += "\n" + helpStyle.Render(helpText)

	return s
}
