use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
    Frame,
};

use crate::models::Email;

use super::header::build_email_header;
use crate::tui::ui::body_viewer;

pub(super) fn render_email_detail(
    frame: &mut Frame,
    area: Rect,
    scroll_offset: &mut usize,
    email: Option<&Email>,
) {
    let Some(email) = email else {
        return;
    };

    let header_lines = build_email_header(email);
    let header_height = header_lines.len() as u16;

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

    body_viewer::render(frame, sections[2], scroll_offset, email.body.as_str());
}
