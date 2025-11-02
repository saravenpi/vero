package email

import (
	"fmt"

	"github.com/emersion/go-imap"
	"github.com/emersion/go-imap/client"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/models"
)

// FetchEmailBody retrieves the full body content of a specific email from the IMAP server.
func FetchEmailBody(cfg *config.IMAPConfig, uid uint32) (string, error) {
	body, _, err := FetchEmailBodyAndAttachments(cfg, uid)
	return body, err
}

// FetchEmailBodyAndAttachments retrieves the full body content and attachments of a specific email from the IMAP server.
func FetchEmailBodyAndAttachments(cfg *config.IMAPConfig, uid uint32) (string, []models.Attachment, error) {
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

	seqSet := new(imap.SeqSet)
	seqSet.AddNum(uid)

	messages := make(chan *imap.Message, 1)
	done := make(chan error, 1)

	section := &imap.BodySectionName{}
	items := []imap.FetchItem{section.FetchItem()}

	go func() {
		done <- c.UidFetch(seqSet, items, messages)
	}()

	var body string
	var attachments []models.Attachment
	for msg := range messages {
		literal := msg.GetBody(section)
		if literal != nil {
			parsedBody, parsedAttachments, err := parseBodyAndAttachments(literal)
			if err != nil {
				return "", nil, fmt.Errorf("failed to parse email body: %w", err)
			}
			body = parsedBody
			attachments = parsedAttachments
			break
		}
	}

	if err := <-done; err != nil {
		return "", nil, fmt.Errorf("fetch error: %w", err)
	}

	if body == "" {
		return "", nil, fmt.Errorf("email body is empty")
	}

	return body, attachments, nil
}
