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

use super::theme::muted_text_style;
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

    let tab_filters = tab_filters(app);
    let tab_index = tab_filters
        .iter()
        .position(|filter| *filter == app.inbox_filter)
        .unwrap_or(0);
    let tabs = Tabs::new(
        tab_filters
            .iter()
            .map(|filter| format!("  {}  ", filter.label()))
            .collect::<Vec<_>>(),
    )
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

    if app.inbox_visible_len() == 0 {
        frame.render_widget(
            Paragraph::new(empty_text(app))
                .block(block)
                .style(muted_text_style())
                .alignment(Alignment::Left),
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
    if app.inbox_search().is_active() {
        return format!(
            " ▼ Inbox ({}/{}) /{} ",
            app.inbox_visible_len(),
            app.inbox_emails.len(),
            app.inbox_search().display_query(),
        );
    }

    format!(" ▼ Inbox ({}) ", app.inbox_emails.len())
}

fn empty_text(app: &App) -> String {
    if app.inbox_search().is_active() && !app.inbox_emails.is_empty() {
        return format!("No emails match /{}", app.inbox_search().display_query());
    }

    if app.inbox_loading && !app.inbox_cache_loaded {
        return "No cached emails".to_string();
    }

    "No emails found".to_string()
}

pub(crate) fn tab_filters(app: &App) -> [InboxFilter; 3] {
    InboxFilter::ordered(InboxFilter::from_str(&app.config.inbox_view))
}
