use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::ui::theme::PRIMARY_COLOR;

pub(super) fn render(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PRIMARY_COLOR))
        .title(Span::styled(
            " ◇ Compose - Opening Editor ",
            Style::default().fg(PRIMARY_COLOR),
        ));
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Opening editor...",
            Style::default().add_modifier(Modifier::ITALIC),
        )),
    ];

    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
}
