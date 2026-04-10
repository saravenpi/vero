mod rows;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, Paragraph, Tabs},
    Frame,
};

use crate::models::{InboxFilter, ViewMode};
use crate::tui::App;

use super::{detail, list, theme::PRIMARY_COLOR};

pub(crate) fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.inbox_view_mode == ViewMode::Detail {
        detail::render(frame, app, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let tab_index = match app.inbox_filter {
        InboxFilter::Unseen => 0,
        InboxFilter::Seen => 1,
        InboxFilter::All => 2,
    };
    let tabs = Tabs::new(vec!["  Unseen  ", "  Seen  ", "  All  "])
        .select(tab_index)
        .style(Style::default().add_modifier(Modifier::DIM))
        .highlight_style(
            Style::default()
                .fg(PRIMARY_COLOR)
                .remove_modifier(Modifier::DIM)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, chunks[0]);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .horizontal_margin(1)
        .split(chunks[1]);

    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(title(app), Style::default().fg(PRIMARY_COLOR)));

    if app.inbox_emails.is_empty() {
        let empty_text = if app.inbox_loading && !app.inbox_cache_loaded {
            "No cached emails"
        } else {
            "No emails found"
        };

        frame.render_widget(
            Paragraph::new(empty_text)
                .block(block)
                .alignment(Alignment::Center),
            inner[0],
        );
        return;
    }

    let available_width = inner[0].width.saturating_sub(4) as usize;
    let items = rows::items(app, available_width);

    let list_widget = List::new(items).block(block);
    list::render_stateful_list(
        frame,
        inner[0],
        list_widget,
        &mut app.inbox_selected,
        &mut app.inbox_list_offset,
    );
}

pub(crate) fn title(app: &App) -> String {
    format!(" ▼ Inbox ({}) ", app.inbox_emails.len())
}
