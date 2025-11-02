package config

import (
	"fmt"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

// VeroConfig represents the main configuration file structure.
type VeroConfig struct {
	Accounts         []Account `yaml:"accounts"`
	DownloadFolder   string    `yaml:"download_folder,omitempty"`
	DefaultInboxView string    `yaml:"default_inbox_view,omitempty"`
}

// Account represents a single email account configuration.
type Account struct {
	Email string     `yaml:"email"`
	IMAP  IMAPConfig `yaml:"imap"`
	SMTP  SMTPConfig `yaml:"smtp"`
}

// IMAPConfig contains IMAP server connection settings.
type IMAPConfig struct {
	User     string `yaml:"user"`
	Password string `yaml:"password"`
	Host     string `yaml:"host"`
	Port     int    `yaml:"port"`
}

// SMTPConfig contains SMTP server connection settings.
type SMTPConfig struct {
	User     string `yaml:"user"`
	Password string `yaml:"password"`
	Host     string `yaml:"host"`
	Port     int    `yaml:"port"`
}

// Load reads and parses the Vero configuration file from ~/.vero.yml.
func Load() (*VeroConfig, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return nil, fmt.Errorf("unable to find home directory: %w", err)
	}

	configPath := filepath.Join(home, ".vero.yml")

	data, err := os.ReadFile(configPath)
	if err != nil {
		return nil, fmt.Errorf("unable to read config file at %s: %w", configPath, err)
	}

	var cfg VeroConfig
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("unable to parse config file: %w", err)
	}

	if len(cfg.Accounts) == 0 {
		return nil, fmt.Errorf("no accounts configured in %s", configPath)
	}

	for i := range cfg.Accounts {
		if cfg.Accounts[i].Email == "" {
			return nil, fmt.Errorf("account at index %d is missing email", i)
		}
		if cfg.Accounts[i].IMAP.Host == "" {
			return nil, fmt.Errorf("account %s is missing IMAP host", cfg.Accounts[i].Email)
		}
		if cfg.Accounts[i].SMTP.Host == "" {
			return nil, fmt.Errorf("account %s is missing SMTP host", cfg.Accounts[i].Email)
		}
		if cfg.Accounts[i].IMAP.Port == 0 {
			cfg.Accounts[i].IMAP.Port = 993
		}
		if cfg.Accounts[i].SMTP.Port == 0 {
			cfg.Accounts[i].SMTP.Port = 465
		}
		if cfg.Accounts[i].IMAP.User == "" {
			cfg.Accounts[i].IMAP.User = cfg.Accounts[i].Email
		}
		if cfg.Accounts[i].SMTP.User == "" {
			cfg.Accounts[i].SMTP.User = cfg.Accounts[i].Email
		}
	}

	if cfg.DownloadFolder == "" {
		cfg.DownloadFolder = filepath.Join(home, "Downloads")
	} else {
		cfg.DownloadFolder = expandPath(cfg.DownloadFolder, home)
	}

	if cfg.DefaultInboxView == "" {
		cfg.DefaultInboxView = "all"
	} else {
		validViews := map[string]bool{"unseen": true, "seen": true, "all": true}
		if !validViews[cfg.DefaultInboxView] {
			return nil, fmt.Errorf("invalid default_inbox_view '%s', must be 'unseen', 'seen', or 'all'", cfg.DefaultInboxView)
		}
	}

	return &cfg, nil
}

func expandPath(path string, home string) string {
	if filepath.IsAbs(path) {
		return path
	}
	if len(path) > 0 && path[0] == '~' {
		if len(path) == 1 {
			return home
		}
		if path[1] == '/' || path[1] == filepath.Separator {
			return filepath.Join(home, path[2:])
		}
	}
	return path
}
