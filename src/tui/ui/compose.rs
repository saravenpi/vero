mod no_editor;
mod opening;
mod preview;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::tui::app::{App, ComposeStep};

pub(crate) fn render(frame: &mut Frame, app: &App, area: Rect) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .margin(1)
        .split(area);

    match app.compose_step {
        ComposeStep::NoEditor => no_editor::render(frame, inner[0]),
        ComposeStep::Editing => opening::render(frame, inner[0]),
        ComposeStep::Preview => preview::render(frame, app, inner[0]),
    }
}
