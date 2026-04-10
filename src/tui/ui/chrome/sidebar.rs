use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::{App, FocusedElement};
use crate::tui::ui::theme::PRIMARY_COLOR;

pub(crate) fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focused == FocusedElement::MenuBar {
        PRIMARY_COLOR
    } else {
        Color::DarkGray
    };

    let sidebar_block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(border_color));

    frame.render_widget(sidebar_block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(area);

    let account_line = app
        .current_account
        .as_ref()
        .map(|a| a.email.as_str())
        .unwrap_or("No account");

    let logo = Paragraph::new(vec![
        Line::from(Span::styled(
            "  Vero",
            Style::default()
                .fg(PRIMARY_COLOR)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", account_line),
            Style::default().add_modifier(Modifier::DIM),
        )),
    ]);
    frame.render_widget(logo, inner[0]);

    let menu_items = App::menu_items();
    let menu_list: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let icon = match index {
                0 => "▼ ",
                1 => "▲ ",
                2 => "✦ ",
                3 => "✎ ",
                _ => "  ",
            };
            let is_selected = index == app.menu_selected;
            let is_focused = app.focused == FocusedElement::MenuBar;

            let style = if is_selected && is_focused {
                Style::default()
                    .bg(PRIMARY_COLOR)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else if is_selected {
                Style::default()
                    .fg(PRIMARY_COLOR)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = if index == 0 && app.inbox_unseen_count > 0 {
                format!(" {}{} ({})", icon, item, app.inbox_unseen_count)
            } else if index == 2 && !app.drafts.is_empty() {
                format!(" {}{} ({})", icon, item, app.drafts.len())
            } else {
                format!(" {}{}", icon, item)
            };

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    frame.render_widget(List::new(menu_list), inner[2]);
}
