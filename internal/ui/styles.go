package ui

import "github.com/charmbracelet/lipgloss"

var (
	titleStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(lipgloss.Color("5")).
			MarginLeft(2).
			MarginTop(1)

	selectedStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("5")).
			Bold(true)

	normalStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("7"))

	helpStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("243")).
			MarginLeft(2).
			MarginTop(1)

	statusStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("3")).
			MarginLeft(2)

	errorStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("1")).
			Bold(true).
			MarginLeft(2)

	inputStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("6"))

	labelStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("4")).
			Bold(true)

	emailHeaderStyle = lipgloss.NewStyle().
				Foreground(lipgloss.Color("5")).
				Bold(true)

	emailBodyStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("7")).
			MarginLeft(2)

	unreadStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("2")).
			Bold(true)
)
