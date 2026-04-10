use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::App;
use crate::{models::Email, tui::app::Screen};

use super::{
    theme::PRIMARY_COLOR,
    utils::{display_subject, sanitize_email_body},
};

pub(crate) fn render(frame: &mut Frame, app: &App, area: Rect, emails: &[Email], selected: usize) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .margin(1)
        .split(area);

    if selected >= emails.len() {
        return;
    }

    let email = &emails[selected];
    let (subject_text, has_empty_subject) = display_subject(email.subject.as_str());
    let header_style = if has_empty_subject {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(PRIMARY_COLOR)
    };
    let header_block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(format!(" {} ", subject_text), header_style));

    let unknown = String::from("Unknown");
    let to = email.to.as_ref().unwrap_or(&unknown);
    let header_text = vec![
        Line::from(vec![
            Span::styled("From: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(&email.from, Style::default()),
        ]),
        Line::from(vec![
            Span::styled("To: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(to, Style::default()),
        ]),
        Line::from(vec![
            Span::styled("Date: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(&email.date, Style::default()),
        ]),
    ];

    frame.render_widget(Paragraph::new(header_text).block(header_block), inner[0]);

    let scroll_offset = if app.screen == Screen::Inbox {
        app.inbox_scroll_offset
    } else {
        app.sent_scroll_offset
    };
    let body_block = Block::default().borders(Borders::NONE).title(Span::styled(
        "Body (j/k to scroll)".to_string(),
        Style::default().add_modifier(Modifier::DIM),
    ));
    let body = Paragraph::new(sanitize_email_body(email.body.as_str()))
        .block(body_block)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));

    frame.render_widget(body, inner[1]);
}
