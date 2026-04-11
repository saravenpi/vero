use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::{
    app::App,
    ui::{body_viewer, theme::PRIMARY_COLOR},
};

pub(super) fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let mut header_lines = vec![Line::from(vec![
        Span::styled(
            "To: ",
            Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
        ),
        Span::styled(&app.compose_draft.to, Style::default()),
    ])];

    if !app.compose_draft.cc.is_empty() {
        header_lines.push(Line::from(vec![
            Span::styled(
                "CC: ",
                Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
            ),
            Span::styled(&app.compose_draft.cc, Style::default()),
        ]));
    }

    if !app.compose_draft.bcc.is_empty() {
        header_lines.push(Line::from(vec![
            Span::styled(
                "BCC: ",
                Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
            ),
            Span::styled(&app.compose_draft.bcc, Style::default()),
        ]));
    }

    header_lines.push(Line::from(vec![
        Span::styled(
            "Subject: ",
            Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
        ),
        Span::styled(&app.compose_draft.subject, Style::default()),
    ]));

    if !app.compose_draft.attachments.is_empty() {
        header_lines.push(Line::from(vec![
            Span::styled(
                "Attachments: ",
                Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
            ),
            Span::styled(
                format!("{} file(s)", app.compose_draft.attachments.len()),
                Style::default(),
            ),
        ]));
    }

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(header_lines.len() as u16),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(Span::styled("Preview", Style::default().fg(PRIMARY_COLOR))),
        sections[0],
    );
    frame.render_widget(Paragraph::new(header_lines), sections[1]);
    frame.render_widget(
        Paragraph::new("Body:").style(Style::default().add_modifier(Modifier::DIM)),
        sections[2],
    );
    body_viewer::render(
        frame,
        sections[3],
        &mut app.compose_preview_scroll_offset,
        app.compose_draft.body.as_str(),
    );
}
