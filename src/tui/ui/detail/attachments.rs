use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::models::Email;

use super::header::{build_email_header, format_size};
use crate::tui::ui::theme::PRIMARY_COLOR;

pub(super) fn render_attachment_list(
    frame: &mut Frame,
    area: Rect,
    email: Option<&Email>,
    attachment_selected: usize,
) {
    let Some(email) = email else {
        return;
    };

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(area);

    frame.render_widget(Paragraph::new(build_email_header(email)), sections[0]);

    let label = if email.attachments.is_empty() {
        "No attachments".to_string()
    } else {
        format!(
            "Attachments ({})  j/k: Navigate  Enter: Download  a: All  Esc: Back",
            email.attachments.len()
        )
    };
    frame.render_widget(
        Paragraph::new(label).style(Style::default().add_modifier(Modifier::DIM)),
        sections[1],
    );

    let mut lines = Vec::new();
    for (index, attachment) in email.attachments.iter().enumerate() {
        let is_selected = index == attachment_selected;
        let marker = if is_selected { ">" } else { " " };
        let size_str = format_size(attachment.size);
        let line_str = format!(
            " {} {}  {}  {}",
            marker, attachment.filename, attachment.content_type, size_str
        );
        let style = if is_selected {
            Style::default().fg(PRIMARY_COLOR)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(line_str, style)));
    }

    frame.render_widget(Paragraph::new(lines), sections[2]);
}
