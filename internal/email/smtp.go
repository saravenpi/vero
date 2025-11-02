package email

import (
	"bytes"
	"crypto/tls"
	"fmt"
	"io"
	"net/smtp"
	"os"
	"strings"
	"time"

	"github.com/emersion/go-message/mail"
	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/models"
)

// SendEmail sends an email using the SMTP server with the provided draft content.
func SendEmail(cfg *config.SMTPConfig, draft models.EmailDraft) error {
	to := parseAddresses(draft.To)
	if draft.CC != "" {
		to = append(to, parseAddresses(draft.CC)...)
	}

	msg := buildMessage(cfg.User, draft)
	addr := fmt.Sprintf("%s:%d", cfg.Host, cfg.Port)

	tlsConfig := &tls.Config{
		ServerName: cfg.Host,
	}

	conn, err := tls.Dial("tcp", addr, tlsConfig)
	if err != nil {
		return fmt.Errorf("failed to connect to SMTP server: %w", err)
	}
	defer conn.Close()

	client, err := smtp.NewClient(conn, cfg.Host)
	if err != nil {
		return fmt.Errorf("failed to create SMTP client: %w", err)
	}
	defer client.Close()

	auth := smtp.PlainAuth("", cfg.User, cfg.Password, cfg.Host)
	if err := client.Auth(auth); err != nil {
		return fmt.Errorf("SMTP authentication failed: %w", err)
	}

	if err := client.Mail(cfg.User); err != nil {
		return fmt.Errorf("failed to set sender: %w", err)
	}

	for _, recipient := range to {
		if err := client.Rcpt(recipient); err != nil {
			return fmt.Errorf("failed to add recipient %s: %w", recipient, err)
		}
	}

	w, err := client.Data()
	if err != nil {
		return fmt.Errorf("failed to initialize data command: %w", err)
	}

	_, err = w.Write([]byte(msg))
	if err != nil {
		return fmt.Errorf("failed to write message: %w", err)
	}

	err = w.Close()
	if err != nil {
		return fmt.Errorf("failed to close data writer: %w", err)
	}

	return client.Quit()
}

func parseAddresses(addresses string) []string {
	parts := strings.Split(addresses, ",")
	result := make([]string, 0, len(parts))
	for _, addr := range parts {
		addr = strings.TrimSpace(addr)
		if addr != "" {
			result = append(result, addr)
		}
	}
	return result
}

func buildMessage(from string, draft models.EmailDraft) string {
	var buf bytes.Buffer

	var h mail.Header
	h.SetDate(time.Now())
	h.SetAddressList("From", []*mail.Address{{Address: from}})
	h.SetAddressList("To", parseAddressesForHeader(draft.To))
	if draft.CC != "" {
		h.SetAddressList("Cc", parseAddressesForHeader(draft.CC))
	}
	h.SetSubject(draft.Subject)

	if len(draft.Attachments) == 0 {
		h.SetContentType("text/plain", map[string]string{"charset": "UTF-8"})

		writer, err := mail.CreateWriter(&buf, h)
		if err != nil {
			return buildSimpleMessage(from, draft)
		}

		textWriter, err := writer.CreateInline()
		if err != nil {
			return buildSimpleMessage(from, draft)
		}

		var textHeader mail.InlineHeader
		textHeader.SetContentType("text/plain", map[string]string{"charset": "UTF-8"})

		textPart, err := textWriter.CreatePart(textHeader)
		if err != nil {
			return buildSimpleMessage(from, draft)
		}

		io.WriteString(textPart, draft.Body)
		textPart.Close()
		textWriter.Close()
		writer.Close()

		return buf.String()
	}

	h.SetContentType("multipart/mixed", nil)

	writer, err := mail.CreateWriter(&buf, h)
	if err != nil {
		return buildSimpleMessage(from, draft)
	}

	inlineWriter, err := writer.CreateInline()
	if err != nil {
		return buildSimpleMessage(from, draft)
	}

	var textHeader mail.InlineHeader
	textHeader.SetContentType("text/plain", map[string]string{"charset": "UTF-8"})

	textPart, err := inlineWriter.CreatePart(textHeader)
	if err != nil {
		return buildSimpleMessage(from, draft)
	}

	io.WriteString(textPart, draft.Body)
	textPart.Close()
	inlineWriter.Close()

	for _, attachment := range draft.Attachments {
		if err := addAttachment(writer, attachment); err != nil {
			continue
		}
	}

	writer.Close()

	return buf.String()
}

func addAttachment(writer *mail.Writer, attachment models.Attachment) error {
	data, err := os.ReadFile(attachment.FilePath)
	if err != nil {
		return err
	}

	var attachHeader mail.AttachmentHeader
	attachHeader.SetFilename(attachment.Filename)
	if attachment.ContentType != "" {
		attachHeader.SetContentType(attachment.ContentType, nil)
	} else {
		attachHeader.SetContentType("application/octet-stream", nil)
	}

	attachPart, err := writer.CreateAttachment(attachHeader)
	if err != nil {
		return err
	}

	_, err = attachPart.Write(data)
	if err != nil {
		return err
	}

	return attachPart.Close()
}

func parseAddressesForHeader(addresses string) []*mail.Address {
	parts := strings.Split(addresses, ",")
	result := make([]*mail.Address, 0, len(parts))
	for _, addr := range parts {
		addr = strings.TrimSpace(addr)
		if addr != "" {
			result = append(result, &mail.Address{Address: addr})
		}
	}
	return result
}

func buildSimpleMessage(from string, draft models.EmailDraft) string {
	var msg strings.Builder

	msg.WriteString(fmt.Sprintf("From: %s\r\n", from))
	msg.WriteString(fmt.Sprintf("To: %s\r\n", draft.To))
	if draft.CC != "" {
		msg.WriteString(fmt.Sprintf("Cc: %s\r\n", draft.CC))
	}
	msg.WriteString(fmt.Sprintf("Subject: %s\r\n", draft.Subject))
	msg.WriteString(fmt.Sprintf("Date: %s\r\n", time.Now().Format(time.RFC1123Z)))
	msg.WriteString("MIME-Version: 1.0\r\n")
	msg.WriteString("Content-Type: text/plain; charset=UTF-8\r\n")
	msg.WriteString("\r\n")
	msg.WriteString(draft.Body)

	return msg.String()
}
