use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};

use crate::tui::App;

use super::theme::PRIMARY_COLOR;

pub(crate) fn render(frame: &mut Frame, app: &App, area: Rect) {
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Min(10),
            Constraint::Percentage(30),
        ])
        .split(area);

    let num_accounts = app.config.accounts.len();
    let list_height = (num_accounts + 4).min(15);

    let horizontal_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Min(50),
            Constraint::Percentage(25),
        ])
        .split(vertical_center[1]);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(list_height as u16),
            Constraint::Min(0),
        ])
        .split(horizontal_center[1]);

    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Vero",
            Style::default()
                .fg(PRIMARY_COLOR)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Select Account",
            Style::default().fg(PRIMARY_COLOR),
        )),
    ]);

    frame.render_widget(title, inner[0]);

    let items: Vec<ListItem> = app
        .config
        .accounts
        .iter()
        .enumerate()
        .map(|(index, account)| {
            let is_selected = index == app.account_selected;
            let style = if is_selected {
                Style::default()
                    .bg(PRIMARY_COLOR)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default()
            };
            let prefix = if is_selected { "▸ " } else { "  " };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(PRIMARY_COLOR)),
                Span::styled(&account.email, style),
            ]);

            ListItem::new(line)
        })
        .collect();

    frame.render_widget(List::new(items), inner[1]);
}
