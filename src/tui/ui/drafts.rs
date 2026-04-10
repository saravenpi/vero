use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::path::Path;

use crate::tui::App;

use super::{
    list,
    theme::PRIMARY_COLOR,
    utils::truncate_with_ellipsis,
};

pub(crate) fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .margin(1)
        .split(area);

    let title = format!(" ✦ Drafts ({}) ", app.drafts.len());
    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(title, Style::default().fg(PRIMARY_COLOR)));

    if app.drafts.is_empty() {
        frame.render_widget(
            Paragraph::new("No drafts found")
                .block(block)
                .alignment(Alignment::Center),
            inner[0],
        );
        return;
    }

    let available_width = inner[0].width.saturating_sub(4) as usize;
    let items: Vec<ListItem> = app
        .drafts
        .iter()
        .enumerate()
        .map(|(index, (path, draft))| {
            let is_selected = index == app.drafts_selected;
            let (bg_color, fg_color, modifier) = list::selection_style(is_selected);
            let subject_max = available_width.saturating_sub(35).max(20);
            let to_max = 30;
            let subject_raw = if draft.subject.is_empty() {
                "(No Subject)".to_string()
            } else {
                draft.subject.clone()
            };
            let subject = truncate_with_ellipsis(subject_raw.as_str(), subject_max);
            let to_raw = if draft.to.is_empty() {
                Path::new(path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            } else {
                format!("To: {}", draft.to)
            };
            let to_text = truncate_with_ellipsis(to_raw.as_str(), to_max);
            let padding = available_width.saturating_sub(subject.len() + to_text.len() + 2);
            let spaces = " ".repeat(padding);

            let line = Line::from(vec![
                Span::styled(" ", Style::default().bg(bg_color).add_modifier(modifier)),
                Span::styled(
                    subject,
                    Style::default()
                        .fg(fg_color)
                        .bg(bg_color)
                        .add_modifier(modifier),
                ),
                Span::styled(spaces, Style::default().bg(bg_color).add_modifier(modifier)),
                Span::styled(
                    to_text,
                    Style::default()
                        .fg(fg_color)
                        .bg(bg_color)
                        .add_modifier(if is_selected {
                            modifier
                        } else {
                            modifier | Modifier::DIM
                        }),
                ),
                Span::styled(" ", Style::default().bg(bg_color).add_modifier(modifier)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_widget = List::new(items).block(block);
    list::render_stateful_list(
        frame,
        inner[0],
        list_widget,
        &mut app.drafts_selected,
        &mut app.drafts_list_offset,
    );
}
