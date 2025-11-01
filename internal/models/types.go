package models

import "time"

// Email represents an email message with its metadata and content.
type Email struct {
	From      string    `yaml:"from"`
	To        string    `yaml:"to,omitempty"`
	CC        string    `yaml:"cc,omitempty"`
	Subject   string    `yaml:"subject"`
	Date      string    `yaml:"date"`
	Body      string    `yaml:"body"`
	Timestamp time.Time `yaml:"timestamp"`
}

// Section represents the current section of the application.
type Section int

const (
	SectionMenu Section = iota
	SectionInbox
	SectionSent
	SectionCompose
)

// ViewMode represents the current view mode (list or detail).
type ViewMode int

const (
	ViewList ViewMode = iota
	ViewDetail
)

// InboxFilter represents email filtering options for the inbox.
type InboxFilter int

const (
	FilterUnseen InboxFilter = iota
	FilterSeen
	FilterAll
)

// String returns the string representation of the inbox filter.
func (f InboxFilter) String() string {
	switch f {
	case FilterUnseen:
		return "unseen"
	case FilterSeen:
		return "seen"
	case FilterAll:
		return "all"
	default:
		return "all"
	}
}

// EmailDraft represents an email being composed before sending.
type EmailDraft struct {
	To      string
	CC      string
	Subject string
	Body    string
}
