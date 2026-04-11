use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthChar;

use crate::tui::App;
use crate::{models::Email, tui::app::Screen};

use super::{
    theme::PRIMARY_COLOR,
    utils::{display_subject, sanitize_email_body},
};

pub(crate) fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.screen {
        Screen::Inbox => {
            if app.inbox_show_attachments {
                render_attachment_list(
                    frame,
                    area,
                    &app.inbox_emails,
                    app.inbox_selected,
                    app.inbox_attachment_selected,
                );
            } else {
                render_email_detail(
                    frame,
                    area,
                    &mut app.inbox_scroll_offset,
                    &app.inbox_emails,
                    app.inbox_selected,
                );
            }
        }
        Screen::Sent => render_email_detail(
            frame,
            area,
            &mut app.sent_scroll_offset,
            &app.sent_emails,
            app.sent_selected,
        ),
        _ => {}
    }
}

fn render_attachment_list(
    frame: &mut Frame,
    area: Rect,
    emails: &[Email],
    selected: usize,
    attachment_selected: usize,
) {
    if selected >= emails.len() {
        return;
    }

    let email = &emails[selected];

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(area);

    let (subject_text, has_empty_subject) = display_subject(email.subject.as_str());
    let header_style = if has_empty_subject {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(PRIMARY_COLOR)
    };
    let unknown = String::from("Unknown");
    let to = email.to.as_ref().unwrap_or(&unknown);
    let header_text = vec![
        Line::from(Span::styled(format!("{} ", subject_text), header_style)),
        Line::from(vec![
            Span::styled("From: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(&email.from, Style::default()),
        ]),
        Line::from(vec![
            Span::styled("To: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(to, Style::default()),
        ]),
        Line::from(vec![
            Span::styled("Date: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(&email.date, Style::default()),
        ]),
        Line::from(vec![
            Span::styled("Attachments: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(
                email.attachments.len().to_string(),
                Style::default(),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(header_text), sections[0]);

    let label = if email.attachments.is_empty() {
        "No attachments".to_string()
    } else {
        format!(
            "Attachments ({})  j/k: Navigate  Enter: Download  a: All  Esc: Back",
            email.attachments.len()
        )
    };
    frame.render_widget(
        Paragraph::new(label).style(Style::default().add_modifier(Modifier::DIM)),
        sections[1],
    );

    let mut lines = Vec::new();
    for (i, attachment) in email.attachments.iter().enumerate() {
        let is_selected = i == attachment_selected;
        let marker = if is_selected { ">" } else { " " };
        let size_str = format_size(attachment.size);
        let line_str = format!(
            " {} {}  {}  {}",
            marker, attachment.filename, attachment.content_type, size_str
        );
        let style = if is_selected {
            Style::default().fg(PRIMARY_COLOR)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(line_str, style)));
    }

    frame.render_widget(Paragraph::new(lines), sections[2]);
}

fn format_size(bytes: i64) -> String {
    if bytes <= 0 {
        return String::new();
    }
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.1} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn render_email_detail(
    frame: &mut Frame,
    area: Rect,
    scroll_offset: &mut usize,
    emails: &[Email],
    selected: usize,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(area);

    if selected >= emails.len() {
        return;
    }

    let email = &emails[selected];
    let (subject_text, has_empty_subject) = display_subject(email.subject.as_str());
    let header_style = if has_empty_subject {
        Style::default().add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(PRIMARY_COLOR)
    };
    let unknown = String::from("Unknown");
    let to = email.to.as_ref().unwrap_or(&unknown);
    let mut header_text = vec![
        Line::from(Span::styled(format!("{} ", subject_text), header_style)),
        Line::from(vec![
            Span::styled("From: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(&email.from, Style::default()),
        ]),
        Line::from(vec![
            Span::styled("To: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(to, Style::default()),
        ]),
        Line::from(vec![
            Span::styled("Date: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(&email.date, Style::default()),
        ]),
    ];

    if !email.attachments.is_empty() {
        header_text.push(Line::from(vec![
            Span::styled("Attachments: ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(
                email.attachments.len().to_string(),
                Style::default(),
            ),
        ]));
    }

    frame.render_widget(Paragraph::new(header_text), sections[0]);

    frame.render_widget(
        Paragraph::new("Body (j/k to scroll)").style(Style::default().add_modifier(Modifier::DIM)),
        sections[1],
    );

    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(sections[2]);
    let body_text_area = body_layout[0];
    let scrollbar_area = body_layout[1];

    let body_text = sanitize_email_body(email.body.as_str());
    let body_line_count = rendered_body_line_count(body_text.as_str(), body_text_area.width);
    let viewport_height = body_text_area.height as usize;
    let max_scroll = body_line_count.saturating_sub(viewport_height);
    *scroll_offset = (*scroll_offset).min(max_scroll);

    let body = Paragraph::new(body_text.as_str())
        .wrap(Wrap { trim: false })
        .scroll(((*scroll_offset).min(u16::MAX as usize) as u16, 0));
    frame.render_widget(body, body_text_area);

    if scrollbar_area.width == 0 || scrollbar_area.height == 0 {
        return;
    }

    let scrollbar_content_length = if max_scroll == 0 { 1 } else { max_scroll + 1 };
    let mut scrollbar_state = ScrollbarState::new(scrollbar_content_length).position(*scroll_offset);

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .track_style(Style::default().add_modifier(Modifier::DIM))
            .thumb_symbol("█")
            .thumb_style(Style::default().fg(PRIMARY_COLOR)),
        scrollbar_area,
        &mut scrollbar_state,
    );
}

fn rendered_body_line_count(body: &str, width: u16) -> usize {
    let width = width as usize;
    if width == 0 {
        return 0;
    }

    body.split('\n')
        .map(|line| wrapped_line_count(line, width))
        .sum()
}

fn wrapped_line_count(line: &str, width: usize) -> usize {
    if line.is_empty() {
        return 1;
    }

    let mut lines = 1;
    let mut current_width = 0;
    let mut run = String::new();
    let mut run_is_whitespace = None;

    for ch in line.chars() {
        let is_whitespace = ch.is_whitespace();
        if run_is_whitespace == Some(is_whitespace) || run_is_whitespace.is_none() {
            run.push(ch);
            run_is_whitespace = Some(is_whitespace);
            continue;
        }

        push_run(
            run.as_str(),
            run_is_whitespace.unwrap_or(false),
            width,
            &mut lines,
            &mut current_width,
        );
        run.clear();
        run.push(ch);
        run_is_whitespace = Some(is_whitespace);
    }

    if !run.is_empty() {
        push_run(
            run.as_str(),
            run_is_whitespace.unwrap_or(false),
            width,
            &mut lines,
            &mut current_width,
        );
    }

    lines
}

fn push_run(
    run: &str,
    is_whitespace: bool,
    width: usize,
    lines: &mut usize,
    current_width: &mut usize,
) {
    let run_width = run
        .chars()
        .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(0))
        .sum::<usize>();

    if run_width == 0 {
        return;
    }

    if !is_whitespace && *current_width > 0 && *current_width + run_width > width {
        *lines += 1;
        *current_width = 0;
    }

    if run_width <= width.saturating_sub(*current_width) {
        *current_width += run_width;
        return;
    }

    for ch in run.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if ch_width == 0 {
            continue;
        }

        if *current_width + ch_width > width {
            *lines += 1;
            *current_width = 0;
        }

        *current_width += ch_width;
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::{
        models::ViewMode,
        tui::{
            app::Screen,
            test_support::{test_app, test_email},
        },
    };
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn short_email_body_clamps_scroll_offset() {
        let backend = TestBackend::new(60, 16);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = test_app();
        let mut email = test_email(1);
        email.body = "short body".to_string();
        app.screen = Screen::Inbox;
        app.inbox_view_mode = ViewMode::Detail;
        app.inbox_emails = vec![email];
        app.inbox_scroll_offset = 5;

        terminal
            .draw(|frame| render(frame, &mut app, frame.area()))
            .unwrap();

        assert_eq!(app.inbox_scroll_offset, 0);
    }

    #[test]
    fn detail_view_renders_scrollbar_symbols() {
        let backend = TestBackend::new(60, 16);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = test_app();
        let mut email = test_email(1);
        email.body = (1..=40)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        app.screen = Screen::Inbox;
        app.inbox_view_mode = ViewMode::Detail;
        app.inbox_emails = vec![email];
        app.inbox_scroll_offset = 8;

        terminal
            .draw(|frame| render(frame, &mut app, frame.area()))
            .unwrap();

        let buffer = terminal.backend().buffer();
        let has_thumb = buffer.content().iter().any(|cell| cell.symbol() == "█");
        let has_track = buffer.content().iter().any(|cell| cell.symbol() == "│");

        assert!(has_thumb);
        assert!(has_track);
    }
}
