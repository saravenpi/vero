use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::models::Email;

use crate::tui::ui::{theme::PRIMARY_COLOR, utils::display_subject};

pub(super) fn build_email_header(email: &Email) -> Vec<Line<'_>> {
    let (subject_text, has_empty_subject) = display_subject(email.subject.as_str());
    let header_style = if has_empty_subject {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(PRIMARY_COLOR)
    };
    let to = email.to.as_deref().unwrap_or("Unknown");

    let mut lines = vec![
        Line::from(Span::styled(format!("{} ", subject_text), header_style)),
        Line::from(vec![
            Span::styled("From: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(email.from.as_str(), Style::default()),
        ]),
        Line::from(vec![
            Span::styled("To: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(to, Style::default()),
        ]),
        Line::from(vec![
            Span::styled("Date: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(email.date.as_str(), Style::default()),
        ]),
    ];

    if !email.attachments.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                "Attachments: ",
                Style::default().add_modifier(Modifier::DIM),
            ),
            Span::styled(email.attachments.len().to_string(), Style::default()),
        ]));
    }

    lines
}

pub(super) fn format_size(bytes: i64) -> String {
    if bytes <= 0 {
        return String::new();
    }
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.1} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{} B", bytes)
    }
}
