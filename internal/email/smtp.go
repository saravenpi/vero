package email

import (
	"crypto/tls"
	"fmt"
	"net/smtp"
	"strings"
	"time"

	"github.com/saravenpi/vero/internal/config"
	"github.com/saravenpi/vero/internal/models"
)

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
