mod account_selection;
mod body_viewer;
mod chrome;
mod compose;
mod detail;
mod drafts;
mod inbox;
mod list;
pub(super) mod quote;
mod sent;
mod signatures;
mod theme;
mod utils;

#[cfg(test)]
mod tests;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Clear,
    Frame,
};

use super::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &mut App) {
    if app.screen == Screen::AccountSelection {
        account_selection::render(frame, app, frame.area());
        return;
    }

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .split(frame.area());

    chrome::render_sidebar(frame, app, main_layout[0]);

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(main_layout[1]);

    render_main_content(frame, app, right_layout[0]);
    chrome::render_footer(frame, app, right_layout[1]);
}

fn render_main_content(frame: &mut Frame, app: &mut App, area: Rect) {
    frame.render_widget(Clear, area);

    match app.screen {
        Screen::Inbox => inbox::render(frame, app, area),
        Screen::Drafts => drafts::render(frame, app, area),
        Screen::Sent => sent::render(frame, app, area),
        Screen::Compose => compose::render(frame, app, area),
        Screen::Signatures => signatures::render(frame, app, area),
        Screen::AccountSelection => {}
    }
}
