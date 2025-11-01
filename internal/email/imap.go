package email

import (
	"fmt"
	"io"
	"io/ioutil"
	"strings"

	"github.com/emersion/go-imap"
	"github.com/emersion/go-imap/client"
	"github.com/emersion/go-message/mail"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/models"
)

func FetchEmails(cfg *config.IMAPConfig, filter models.InboxFilter) ([]models.Email, error) {
	c, err := client.DialTLS(fmt.Sprintf("%s:%d", cfg.Host, cfg.Port), nil)
	if err != nil {
		return nil, fmt.Errorf("failed to connect: %w", err)
	}
	defer c.Logout()

	if err := c.Login(cfg.User, cfg.Password); err != nil {
		return nil, fmt.Errorf("failed to login: %w", err)
	}

	mbox, err := c.Select("INBOX", false)
	if err != nil {
		return nil, fmt.Errorf("failed to select INBOX: %w", err)
	}

	if mbox.Messages == 0 {
		return []models.Email{}, nil
	}

	criteria := imap.NewSearchCriteria()
	switch filter {
	case models.FilterUnseen:
		criteria.WithoutFlags = []string{imap.SeenFlag}
	case models.FilterSeen:
		criteria.WithFlags = []string{imap.SeenFlag}
	}

	seqNums, err := c.Search(criteria)
	if err != nil {
		return nil, fmt.Errorf("failed to search: %w", err)
	}

	if len(seqNums) == 0 {
		return []models.Email{}, nil
	}

	seqSet := new(imap.SeqSet)
	seqSet.AddNum(seqNums...)

	messages := make(chan *imap.Message, 10)
	done := make(chan error, 1)

	items := []imap.FetchItem{imap.FetchEnvelope}

	go func() {
		done <- c.Fetch(seqSet, items, messages)
	}()

	emails := []models.Email{}
	for msg := range messages {
		email := parseEnvelope(msg)
		emails = append(emails, email)
	}

	if err := <-done; err != nil {
		return nil, fmt.Errorf("fetch error: %w", err)
	}

	reverseEmails(emails)
	return emails, nil
}

func parseEnvelope(msg *imap.Message) models.Email {
	email := models.Email{}

	if msg.Envelope != nil {
		if len(msg.Envelope.From) > 0 {
			email.From = formatAddress(msg.Envelope.From[0])
		}
		email.Subject = msg.Envelope.Subject
		email.Date = msg.Envelope.Date.Format("Mon, 02 Jan 2006 15:04:05 -0700")
	}

	return email
}

func parseBody(r io.Reader) (string, error) {
	data, err := ioutil.ReadAll(r)
	if err != nil {
		return "", fmt.Errorf("failed to read email data: %w", err)
	}

	if len(data) == 0 {
		return "", fmt.Errorf("email data is empty")
	}

	mr, err := mail.CreateReader(strings.NewReader(string(data)))
	if err != nil {
		return string(data), nil
	}

	plainText, htmlText, hasContent := extractTextFromParts(mr)

	if plainText != "" {
		return plainText, nil
	}

	if htmlText != "" {
		return htmlText, nil
	}

	if !hasContent {
		return "", fmt.Errorf("no text content found in email (only attachments or unsupported content types)")
	}

	return "", fmt.Errorf("email body could not be extracted")
}

func extractTextFromParts(mr *mail.Reader) (plainText string, htmlText string, hasContent bool) {
	var plain strings.Builder
	var html strings.Builder

	for {
		part, err := mr.NextPart()
		if err == io.EOF {
			break
		}
		if err != nil {
			break
		}

		switch h := part.Header.(type) {
		case *mail.InlineHeader:
			contentType, _, _ := h.ContentType()

			if strings.HasPrefix(contentType, "text/plain") {
				b, err := io.ReadAll(part.Body)
				if err == nil && len(b) > 0 {
					plain.Write(b)
					hasContent = true
				}
			} else if strings.HasPrefix(contentType, "text/html") {
				b, err := io.ReadAll(part.Body)
				if err == nil && len(b) > 0 {
					html.Write(b)
					hasContent = true
				}
			} else if strings.HasPrefix(contentType, "multipart/") {
				nestedReader, err := mail.CreateReader(part.Body)
				if err == nil {
					nestedPlain, nestedHTML, nestedHasContent := extractTextFromParts(nestedReader)
					if nestedHasContent {
						if nestedPlain != "" {
							plain.WriteString(nestedPlain)
						}
						if nestedHTML != "" {
							html.WriteString(nestedHTML)
						}
						hasContent = true
					}
				}
			}

		case *mail.AttachmentHeader:
			contentType, _, _ := h.ContentType()
			if strings.HasPrefix(contentType, "multipart/") {
				nestedReader, err := mail.CreateReader(part.Body)
				if err == nil {
					nestedPlain, nestedHTML, nestedHasContent := extractTextFromParts(nestedReader)
					if nestedHasContent {
						if nestedPlain != "" {
							plain.WriteString(nestedPlain)
						}
						if nestedHTML != "" {
							html.WriteString(nestedHTML)
						}
						hasContent = true
					}
				}
			}
		}
	}

	return plain.String(), html.String(), hasContent
}

func formatAddress(addr *imap.Address) string {
	if addr.PersonalName != "" {
		return fmt.Sprintf("%s <%s@%s>", addr.PersonalName, addr.MailboxName, addr.HostName)
	}
	return fmt.Sprintf("%s@%s", addr.MailboxName, addr.HostName)
}

func reverseEmails(emails []models.Email) {
	for i := 0; i < len(emails)/2; i++ {
		j := len(emails) - i - 1
		emails[i], emails[j] = emails[j], emails[i]
	}
}
