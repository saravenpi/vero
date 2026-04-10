use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::ListItem,
};

use crate::tui::{
    ui::{
        list,
        utils::{display_subject, display_width, subject_modifier, truncate_with_ellipsis},
    },
    App,
};

pub(super) fn items(app: &App, available_width: usize) -> Vec<ListItem<'static>> {
    app.inbox_emails
        .iter()
        .enumerate()
        .map(|(index, email)| {
            let is_selected = index == app.inbox_selected;
            let (bg_color, fg_color, modifier) = list::selection_style(is_selected);
            let subject_max = available_width.saturating_sub(30).max(20);
            let from_max = 25;
            let (subject_text, has_empty_subject) = display_subject(email.subject.as_str());
            let subject = truncate_with_ellipsis(subject_text, subject_max);
            let from = truncate_with_ellipsis(email.from.as_str(), from_max);
            let padding =
                available_width.saturating_sub(display_width(&subject) + display_width(&from) + 2);
            let spaces = " ".repeat(padding);

            let line = Line::from(vec![
                Span::styled(" ", Style::default().bg(bg_color).add_modifier(modifier)),
                Span::styled(
                    subject,
                    Style::default()
                        .fg(fg_color)
                        .bg(bg_color)
                        .add_modifier(subject_modifier(modifier, email.is_seen, has_empty_subject)),
                ),
                Span::styled(spaces, Style::default().bg(bg_color).add_modifier(modifier)),
                Span::styled(
                    from,
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
        .collect()
}
