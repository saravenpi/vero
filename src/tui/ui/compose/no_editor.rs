use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::ui::theme::{ERROR_COLOR, SUCCESS_COLOR};

pub(super) fn render(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ERROR_COLOR))
        .title(Span::styled(
            " ◇ Compose - No Editor Configured ",
            Style::default().fg(ERROR_COLOR),
        ));

    let error_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "No editor configured!",
            Style::default()
                .fg(ERROR_COLOR)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "To compose emails, add an editor to ~/.vero.yml:",
            Style::default(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  editor: vim",
            Style::default().add_modifier(Modifier::DIM),
        )),
        Line::from(Span::styled(
            "  editor: nano",
            Style::default().add_modifier(Modifier::DIM),
        )),
        Line::from(Span::styled(
            "  editor: \"nvim -c 'startinsert'\"",
            Style::default().add_modifier(Modifier::DIM),
        )),
        Line::from(Span::styled(
            "  editor: emacs",
            Style::default().add_modifier(Modifier::DIM),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to return to menu.",
            Style::default()
                .fg(SUCCESS_COLOR)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    frame.render_widget(
        Paragraph::new(error_text)
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
}
