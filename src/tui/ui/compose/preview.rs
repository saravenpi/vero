use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::{
    app::App,
    ui::theme::{PRIMARY_COLOR, SUCCESS_COLOR},
};

pub(super) fn render(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PRIMARY_COLOR))
        .title(Span::styled(
            " ◇ Compose - Preview ",
            Style::default().fg(PRIMARY_COLOR),
        ));

    let mut preview_lines = vec![Line::from(vec![
        Span::styled(
            "To: ",
            Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
        ),
        Span::styled(&app.compose_draft.to, Style::default()),
    ])];

    if !app.compose_draft.cc.is_empty() {
        preview_lines.push(Line::from(vec![
            Span::styled(
                "CC: ",
                Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
            ),
            Span::styled(&app.compose_draft.cc, Style::default()),
        ]));
    }

    if !app.compose_draft.bcc.is_empty() {
        preview_lines.push(Line::from(vec![
            Span::styled(
                "BCC: ",
                Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
            ),
            Span::styled(&app.compose_draft.bcc, Style::default()),
        ]));
    }

    preview_lines.push(Line::from(vec![
        Span::styled(
            "Subject: ",
            Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
        ),
        Span::styled(&app.compose_draft.subject, Style::default()),
    ]));

    if !app.compose_draft.attachments.is_empty() {
        preview_lines.push(Line::from(vec![
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

    preview_lines.push(Line::from(""));
    preview_lines.push(Line::from(app.compose_draft.body.as_str()));
    preview_lines.push(Line::from(""));

    if !app.is_sending_email {
        preview_lines.push(Line::from(Span::styled(
            "Enter: Send  |  e: Edit again  |  ESC: Cancel",
            Style::default()
                .fg(SUCCESS_COLOR)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    frame.render_widget(
        Paragraph::new(preview_lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}
