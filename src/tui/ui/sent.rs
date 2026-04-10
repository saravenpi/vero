use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::models::ViewMode;
use crate::tui::App;

use super::{
    detail, list,
    theme::PRIMARY_COLOR,
    utils::{display_subject, display_width, subject_modifier, truncate_with_ellipsis},
};

pub(crate) fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.sent_view_mode == ViewMode::Detail {
        detail::render(frame, app, area, &app.sent_emails, app.sent_selected);
        return;
    }

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .margin(1)
        .split(area);

    let title = format!(" ▲ Sent ({}) ", app.sent_emails.len());
    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(title, Style::default().fg(PRIMARY_COLOR)));

    if app.sent_emails.is_empty() {
        frame.render_widget(
            Paragraph::new("No sent emails found")
                .block(block)
                .alignment(Alignment::Center),
            inner[0],
        );
        return;
    }

    let available_width = inner[0].width.saturating_sub(4) as usize;
    let items: Vec<ListItem> = app
        .sent_emails
        .iter()
        .enumerate()
        .map(|(index, email)| {
            let is_selected = index == app.sent_selected;
            let (bg_color, fg_color, modifier) = list::selection_style(is_selected);
            let unknown = String::from("Unknown");
            let to = email.to.as_ref().unwrap_or(&unknown);
            let subject_max = available_width.saturating_sub(35).max(20);
            let to_max = 30;
            let (subject_text, has_empty_subject) = display_subject(email.subject.as_str());
            let subject = truncate_with_ellipsis(subject_text, subject_max);
            let to_value = format!("To: {}", to);
            let to_text = truncate_with_ellipsis(to_value.as_str(), to_max);
            let padding = available_width
                .saturating_sub(display_width(&subject) + display_width(&to_text) + 2);
            let spaces = " ".repeat(padding);

            let line = Line::from(vec![
                Span::styled(" ", Style::default().bg(bg_color).add_modifier(modifier)),
                Span::styled(
                    subject,
                    Style::default()
                        .fg(fg_color)
                        .bg(bg_color)
                        .add_modifier(subject_modifier(modifier, false, has_empty_subject)),
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
        &mut app.sent_selected,
        &mut app.sent_list_offset,
    );
}
