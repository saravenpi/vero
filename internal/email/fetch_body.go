package email

import (
	"fmt"

	"github.com/emersion/go-imap"
	"github.com/emersion/go-imap/client"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/models"
)

// FetchEmailBody retrieves the full body content of a specific email from the IMAP server.
func FetchEmailBody(cfg *config.IMAPConfig, from, subject string) (string, error) {
	body, _, err := FetchEmailBodyAndAttachments(cfg, from, subject)
	return body, err
}

// FetchEmailBodyAndAttachments retrieves the full body content and attachments of a specific email from the IMAP server.
func FetchEmailBodyAndAttachments(cfg *config.IMAPConfig, from, subject string) (string, []models.Attachment, error) {
	c, err := client.DialTLS(fmt.Sprintf("%s:%d", cfg.Host, cfg.Port), nil)
	if err != nil {
		return "", nil, fmt.Errorf("failed to connect: %w", err)
	}
	defer c.Logout()

	if err := c.Login(cfg.User, cfg.Password); err != nil {
		return "", nil, fmt.Errorf("failed to login: %w", err)
	}

	if _, err := c.Select("INBOX", false); err != nil {
		return "", nil, fmt.Errorf("failed to select INBOX: %w", err)
	}

	criteria := imap.NewSearchCriteria()
	criteria.Header.Set("From", from)
	criteria.Header.Set("Subject", subject)

	seqNums, err := c.Search(criteria)
	if err != nil || len(seqNums) == 0 {
		criteria = imap.NewSearchCriteria()
		seqNums, err = c.Search(criteria)
		if err != nil {
			return "", nil, fmt.Errorf("failed to search: %w", err)
		}
	}

	if len(seqNums) == 0 {
		return "", nil, fmt.Errorf("email not found")
	}

	seqSet := new(imap.SeqSet)
	seqSet.AddNum(seqNums...)

	messages := make(chan *imap.Message, 10)
	done := make(chan error, 1)

	section := &imap.BodySectionName{}
	items := []imap.FetchItem{imap.FetchEnvelope, section.FetchItem()}

	go func() {
		done <- c.Fetch(seqSet, items, messages)
	}()

	var body string
	var attachments []models.Attachment
	var parseErr error
	for msg := range messages {
		if msg.Envelope != nil {
			if len(msg.Envelope.From) > 0 && msg.Envelope.Subject == subject {
				literal := msg.GetBody(section)
				if literal != nil {
					parsedBody, parsedAttachments, err := parseBodyAndAttachments(literal)
					if err != nil {
						parseErr = err
						continue
					}
					body = parsedBody
					attachments = parsedAttachments
					break
				}
			}
		}
	}

	if err := <-done; err != nil {
		return "", nil, fmt.Errorf("fetch error: %w", err)
	}

	if body == "" && parseErr != nil {
		return "", nil, fmt.Errorf("failed to parse email body: %w", parseErr)
	}

	if body == "" {
		return "", nil, fmt.Errorf("email body is empty")
	}

	return body, attachments, nil
}
