mod builder;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
    Frame,
};

use crate::models::Email;
use crate::tui::ui::body_viewer;
use crate::tui::ui::quote::strip_quoted_content;

use super::header::build_email_header;

pub(super) fn render_email_detail(
    frame: &mut Frame,
    area: Rect,
    scroll_offset: &mut usize,
    email: Option<&Email>,
    collapse_quotes: bool,
) {
    let Some(email) = email else {
        return;
    };

    let header_lines = build_email_header(email);
    let header_height = header_lines.len() as u16;

    let body_text = if collapse_quotes {
        let (stripped, _) = strip_quoted_content(&email.body);
        builder::plain(stripped)
    } else {
        builder::styled(&email.body)
    };

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(area);

    frame.render_widget(Paragraph::new(header_lines), sections[0]);
    frame.render_widget(
        Paragraph::new("Body:").style(Style::default().add_modifier(Modifier::DIM)),
        sections[1],
    );
    body_viewer::render(frame, sections[2], scroll_offset, body_text);
}
