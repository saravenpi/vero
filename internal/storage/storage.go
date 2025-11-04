package storage

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/saravenpi/vero/internal/models"
	"gopkg.in/yaml.v3"
)

const (
	veroDir    = ".vero"
	seenDir    = "seen"
	sentDir    = "sent"
	timeFormat = "2006-01-02-150405"
)

// GetVeroDir returns the path to the Vero data directory in the user's home.
func GetVeroDir() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(home, veroDir), nil
}

// GetAccountDir returns the path to a specific account's data directory.
func GetAccountDir(accountEmail string) (string, error) {
	veroPath, err := GetVeroDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(veroPath, accountEmail), nil
}

func ensureDir(path string) error {
	return os.MkdirAll(path, 0755)
}

func sanitizeEmail(email string) string {
	replacer := strings.NewReplacer(
		"@", "_",
		".", "_",
		"+", "_",
		" ", "_",
	)
	return replacer.Replace(email)
}

// SaveSeenEmail persists a viewed email to the account's seen directory.
func SaveSeenEmail(accountEmail string, email models.Email) error {
	accountPath, err := GetAccountDir(accountEmail)
	if err != nil {
		return err
	}

	seenPath := filepath.Join(accountPath, seenDir)
	if err := ensureDir(seenPath); err != nil {
		return err
	}

	timestamp := time.Now()
	filename := fmt.Sprintf("%s-%s.yml",
		timestamp.Format(timeFormat),
		sanitizeEmail(email.From))

	filePath := filepath.Join(seenPath, filename)

	email.Timestamp = timestamp

	data, err := yaml.Marshal(&email)
	if err != nil {
		return err
	}

	return os.WriteFile(filePath, data, 0644)
}

// SaveSentEmail persists a sent email to the account's sent directory.
func SaveSentEmail(accountEmail string, email models.Email) error {
	accountPath, err := GetAccountDir(accountEmail)
	if err != nil {
		return err
	}

	sentPath := filepath.Join(accountPath, sentDir)
	if err := ensureDir(sentPath); err != nil {
		return err
	}

	timestamp := time.Now()
	filename := fmt.Sprintf("%s-%s.yml",
		timestamp.Format(timeFormat),
		sanitizeEmail(email.To))

	filePath := filepath.Join(sentPath, filename)

	email.From = "Me"
	email.Timestamp = timestamp

	data, err := yaml.Marshal(&email)
	if err != nil {
		return err
	}

	return os.WriteFile(filePath, data, 0644)
}

func loadEmailsFromDir(dirPath string) ([]models.Email, error) {
	if err := ensureDir(dirPath); err != nil {
		return nil, err
	}

	files, err := os.ReadDir(dirPath)
	if err != nil {
		return nil, err
	}

	var emails []models.Email
	for _, file := range files {
		if file.IsDir() || !strings.HasSuffix(file.Name(), ".yml") {
			continue
		}

		filePath := filepath.Join(dirPath, file.Name())
		data, err := os.ReadFile(filePath)
		if err != nil {
			continue
		}

		var email models.Email
		if err := yaml.Unmarshal(data, &email); err != nil {
			continue
		}

		emails = append(emails, email)
	}

	sort.Slice(emails, func(i, j int) bool {
		return emails[i].Timestamp.After(emails[j].Timestamp)
	})

	return emails, nil
}

// LoadSeenEmails retrieves all viewed emails for the specified account.
func LoadSeenEmails(accountEmail string) ([]models.Email, error) {
	accountPath, err := GetAccountDir(accountEmail)
	if err != nil {
		return nil, err
	}

	seenPath := filepath.Join(accountPath, seenDir)
	return loadEmailsFromDir(seenPath)
}

// LoadSentEmails retrieves all sent emails for the specified account.
func LoadSentEmails(accountEmail string) ([]models.Email, error) {
	accountPath, err := GetAccountDir(accountEmail)
	if err != nil {
		return nil, err
	}

	sentPath := filepath.Join(accountPath, sentDir)
	return loadEmailsFromDir(sentPath)
}

// IsEmailSeen checks if an email has already been viewed by the account.
func IsEmailSeen(accountEmail, from, subject string) (bool, error) {
	seenEmails, err := LoadSeenEmails(accountEmail)
	if err != nil {
		return false, err
	}

	for _, email := range seenEmails {
		if email.From == from && email.Subject == subject {
			return true, nil
		}
	}

	return false, nil
}

// DeleteSeenEmail removes a seen email from local storage.
func DeleteSeenEmail(accountEmail string, email models.Email) error {
	accountPath, err := GetAccountDir(accountEmail)
	if err != nil {
		return err
	}

	seenPath := filepath.Join(accountPath, seenDir)

	files, err := os.ReadDir(seenPath)
	if err != nil {
		return err
	}

	for _, file := range files {
		if file.IsDir() || !strings.HasSuffix(file.Name(), ".yml") {
			continue
		}

		filePath := filepath.Join(seenPath, file.Name())
		data, err := os.ReadFile(filePath)
		if err != nil {
			continue
		}

		var storedEmail models.Email
		if err := yaml.Unmarshal(data, &storedEmail); err != nil {
			continue
		}

		if storedEmail.From == email.From && storedEmail.Subject == email.Subject {
			return os.Remove(filePath)
		}
	}

	return nil
}
