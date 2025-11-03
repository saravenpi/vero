package email

import (
	"fmt"
	"io"
	"io/ioutil"
	"os"
	"path/filepath"
	"strings"

	"github.com/emersion/go-imap"
	"github.com/emersion/go-imap/client"
	"github.com/emersion/go-message/mail"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/models"
)

// FetchEmails retrieves emails from the IMAP server based on the specified filter.
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

	items := []imap.FetchItem{imap.FetchEnvelope, imap.FetchUid}

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
		if len(msg.Envelope.To) > 0 {
			toAddrs := make([]string, len(msg.Envelope.To))
			for i, addr := range msg.Envelope.To {
				toAddrs[i] = formatAddress(addr)
			}
			email.To = strings.Join(toAddrs, ", ")
		}
		if len(msg.Envelope.Cc) > 0 {
			ccAddrs := make([]string, len(msg.Envelope.Cc))
			for i, addr := range msg.Envelope.Cc {
				ccAddrs[i] = formatAddress(addr)
			}
			email.CC = strings.Join(ccAddrs, ", ")
		}
		email.Subject = msg.Envelope.Subject
		email.Date = msg.Envelope.Date.Format("Mon, 02 Jan 2006 15:04:05 -0700")
		email.Timestamp = msg.Envelope.Date
	}

	email.UID = msg.Uid

	return email
}

func parseBody(r io.Reader) (string, error) {
	body, _, err := parseBodyAndAttachments(r)
	return body, err
}

func parseBodyAndAttachments(r io.Reader) (string, []models.Attachment, error) {
	data, err := ioutil.ReadAll(r)
	if err != nil {
		return "", nil, fmt.Errorf("failed to read email data: %w", err)
	}

	if len(data) == 0 {
		return "", nil, fmt.Errorf("email data is empty")
	}

	mr, err := mail.CreateReader(strings.NewReader(string(data)))
	if err != nil {
		return string(data), nil, nil
	}

	plainText, htmlText, hasContent, attachments := extractTextAndAttachmentsFromParts(mr)

	if plainText != "" {
		return plainText, attachments, nil
	}

	if htmlText != "" {
		return htmlText, attachments, nil
	}

	if !hasContent {
		return "", attachments, fmt.Errorf("no text content found in email (only attachments or unsupported content types)")
	}

	return "", attachments, fmt.Errorf("email body could not be extracted")
}

func extractTextFromParts(mr *mail.Reader) (plainText string, htmlText string, hasContent bool) {
	plain, html, content, _ := extractTextAndAttachmentsFromParts(mr)
	return plain, html, content
}

func extractTextAndAttachmentsFromParts(mr *mail.Reader) (plainText string, htmlText string, hasContent bool, attachments []models.Attachment) {
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
					nestedPlain, nestedHTML, nestedHasContent, nestedAttachments := extractTextAndAttachmentsFromParts(nestedReader)
					if nestedHasContent {
						if nestedPlain != "" {
							plain.WriteString(nestedPlain)
						}
						if nestedHTML != "" {
							html.WriteString(nestedHTML)
						}
						hasContent = true
					}
					attachments = append(attachments, nestedAttachments...)
				} else {
					io.Copy(io.Discard, part.Body)
				}
			} else {
				io.Copy(io.Discard, part.Body)
			}

		case *mail.AttachmentHeader:
			contentType, _, _ := h.ContentType()
			if strings.HasPrefix(contentType, "multipart/") {
				nestedReader, err := mail.CreateReader(part.Body)
				if err == nil {
					nestedPlain, nestedHTML, nestedHasContent, nestedAttachments := extractTextAndAttachmentsFromParts(nestedReader)
					if nestedHasContent {
						if nestedPlain != "" {
							plain.WriteString(nestedPlain)
						}
						if nestedHTML != "" {
							html.WriteString(nestedHTML)
						}
						hasContent = true
					}
					attachments = append(attachments, nestedAttachments...)
				} else {
					io.Copy(io.Discard, part.Body)
				}
			} else {
				if !isInlineImage(h) {
					attachment := extractAttachment(h, part.Body)
					if attachment != nil {
						attachments = append(attachments, *attachment)
					}
				} else {
					io.Copy(io.Discard, part.Body)
				}
			}
		}
	}

	return plain.String(), html.String(), hasContent, attachments
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

func isInlineImage(header *mail.AttachmentHeader) bool {
	disp, params, _ := header.ContentDisposition()

	if disp == "inline" {
		contentType, _, _ := header.ContentType()
		if strings.HasPrefix(contentType, "image/") {
			return true
		}

		if _, ok := params["filename"]; !ok {
			return true
		}
	}

	contentID := header.Get("Content-ID")
	if contentID != "" && disp == "inline" {
		return true
	}

	return false
}

func extractAttachment(header *mail.AttachmentHeader, body io.Reader) *models.Attachment {
	filename, err := header.Filename()
	if err != nil || filename == "" {
		filename = "unnamed_attachment"
	}

	contentType, _, _ := header.ContentType()

	data, err := io.ReadAll(body)
	if err != nil {
		return nil
	}

	attachDir := filepath.Join(os.Getenv("HOME"), ".vero", "attachments")
	os.MkdirAll(attachDir, 0755)

	safeFilename := strings.ReplaceAll(filename, "/", "_")
	safeFilename = strings.ReplaceAll(safeFilename, "..", "_")
	filePath := filepath.Join(attachDir, safeFilename)

	counter := 1
	originalPath := filePath
	for {
		if _, err := os.Stat(filePath); os.IsNotExist(err) {
			break
		}
		ext := filepath.Ext(originalPath)
		nameWithoutExt := strings.TrimSuffix(originalPath, ext)
		filePath = fmt.Sprintf("%s_%d%s", nameWithoutExt, counter, ext)
		counter++
	}

	if err := os.WriteFile(filePath, data, 0644); err != nil {
		return nil
	}

	return &models.Attachment{
		Filename:    filename,
		ContentType: contentType,
		Size:        int64(len(data)),
		FilePath:    filePath,
	}
}
