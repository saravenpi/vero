use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::App;

use super::theme::PRIMARY_COLOR;

pub(crate) fn render(frame: &mut Frame, app: &App, area: Rect) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .margin(1)
        .split(area);

    let account = app
        .current_account
        .as_ref()
        .map(|a| a.email.as_str())
        .unwrap_or("unknown");

    let title = format!(" ✎ Signature — {} ", account);
    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(title, Style::default().fg(PRIMARY_COLOR)));

    match &app.signature_content {
        None => {
            frame.render_widget(
                Paragraph::new("No signature set. Press 'e' to create one.")
                    .block(block)
                    .alignment(Alignment::Center),
                inner[0],
            );
        }
        Some(sig) => {
            let preview_lines: Vec<Line> = std::iter::once(Line::from(Span::styled(
                "-- ",
                Style::default().add_modifier(Modifier::DIM),
            )))
            .chain(sig.lines().map(|l| Line::from(l.to_string())))
            .collect();

            frame.render_widget(Paragraph::new(preview_lines).block(block), inner[0]);
        }
    }
}
