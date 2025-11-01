package email

import (
	"fmt"

	"github.com/emersion/go-imap"
	"github.com/emersion/go-imap/client"
	"github.com/saravenpi/vero/internal/config"
)

func FetchEmailBody(cfg *config.IMAPConfig, from, subject string) (string, error) {
	c, err := client.DialTLS(fmt.Sprintf("%s:%d", cfg.Host, cfg.Port), nil)
	if err != nil {
		return "", fmt.Errorf("failed to connect: %w", err)
	}
	defer c.Logout()

	if err := c.Login(cfg.User, cfg.Password); err != nil {
		return "", fmt.Errorf("failed to login: %w", err)
	}

	if _, err := c.Select("INBOX", false); err != nil {
		return "", fmt.Errorf("failed to select INBOX: %w", err)
	}

	criteria := imap.NewSearchCriteria()
	criteria.Header.Set("From", from)
	criteria.Header.Set("Subject", subject)

	seqNums, err := c.Search(criteria)
	if err != nil || len(seqNums) == 0 {
		criteria = imap.NewSearchCriteria()
		seqNums, err = c.Search(criteria)
		if err != nil {
			return "", fmt.Errorf("failed to search: %w", err)
		}
	}

	if len(seqNums) == 0 {
		return "", fmt.Errorf("email not found")
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
	var parseErr error
	for msg := range messages {
		if msg.Envelope != nil {
			if len(msg.Envelope.From) > 0 && msg.Envelope.Subject == subject {
				literal := msg.GetBody(section)
				if literal != nil {
					parsedBody, err := parseBody(literal)
					if err != nil {
						parseErr = err
						continue
					}
					body = parsedBody
					break
				}
			}
		}
	}

	if err := <-done; err != nil {
		return "", fmt.Errorf("fetch error: %w", err)
	}

	if body == "" && parseErr != nil {
		return "", fmt.Errorf("failed to parse email body: %w", parseErr)
	}

	if body == "" {
		return "", fmt.Errorf("email body is empty")
	}

	return body, nil
}
