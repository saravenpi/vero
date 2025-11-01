package models

import "time"

type Email struct {
	From      string    `yaml:"from"`
	To        string    `yaml:"to,omitempty"`
	CC        string    `yaml:"cc,omitempty"`
	Subject   string    `yaml:"subject"`
	Date      string    `yaml:"date"`
	Body      string    `yaml:"body"`
	Timestamp time.Time `yaml:"timestamp"`
}

type Section int

const (
	SectionMenu Section = iota
	SectionInbox
	SectionSent
	SectionCompose
)

type ViewMode int

const (
	ViewList ViewMode = iota
	ViewDetail
)

type InboxFilter int

const (
	FilterUnseen InboxFilter = iota
	FilterSeen
	FilterAll
)

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

type EmailDraft struct {
	To      string
	CC      string
	Subject string
	Body    string
}
