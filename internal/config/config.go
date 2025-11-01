package config

import (
	"fmt"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

type VeroConfig struct {
	Accounts []Account `yaml:"accounts"`
}

type Account struct {
	Email string     `yaml:"email"`
	IMAP  IMAPConfig `yaml:"imap"`
	SMTP  SMTPConfig `yaml:"smtp"`
}

type IMAPConfig struct {
	User     string `yaml:"user"`
	Password string `yaml:"password"`
	Host     string `yaml:"host"`
	Port     int    `yaml:"port"`
}

type SMTPConfig struct {
	User     string `yaml:"user"`
	Password string `yaml:"password"`
	Host     string `yaml:"host"`
	Port     int    `yaml:"port"`
}

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

	return &cfg, nil
}
