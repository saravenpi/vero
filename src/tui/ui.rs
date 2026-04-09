use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, ComposeStep, FocusedElement, Screen};
use crate::models::ViewMode;
use std::path::Path;

const PRIMARY_COLOR: Color = Color::Cyan;
const SUCCESS_COLOR: Color = Color::Green;
const ERROR_COLOR: Color = Color::Red;

pub fn render(frame: &mut Frame, app: &App) {
    if app.screen == Screen::AccountSelection {
        render_account_selection(frame, app, frame.area());
        return;
    }

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .split(frame.area());

    render_sidebar(frame, app, main_layout[0]);

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(main_layout[1]);

    render_main_content(frame, app, right_layout[0]);
    render_footer(frame, app, right_layout[1]);
}

fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
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

    let logo = Paragraph::new(vec![
        Line::from(Span::styled(
            "  VERO",
            Style::default()
                .fg(PRIMARY_COLOR)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("  Email Client", Style::default())),
    ]);
    frame.render_widget(logo, inner[0]);

    let menu_items = App::menu_items();
    let menu_list: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let icon = match i {
                0 => "▼ ",
                1 => "▲ ",
                2 => "✦ ",
                _ => "  ",
            };

            let is_selected = i == app.menu_selected;
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

            let content = if i == 0 && app.inbox_unseen_count > 0 {
                format!(" {}{} ({})", icon, item, app.inbox_unseen_count)
            } else if i == 2 && !app.drafts.is_empty() {
                format!(" {}{} ({})", icon, item, app.drafts.len())
            } else {
                format!(" {}{}", icon, item)
            };
            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let menu = List::new(menu_list);
    frame.render_widget(menu, inner[2]);
}

fn render_main_content(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(Clear, area);

    match app.screen {
        Screen::Inbox => render_inbox_screen(frame, app, area),
        Screen::Drafts => render_drafts_screen(frame, app, area),
        Screen::Sent => render_sent_screen(frame, app, area),
        Screen::Compose => render_compose_screen(frame, app, area),
        Screen::AccountSelection => {}
    }
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.screen {
        Screen::AccountSelection => "↑/↓: Navigate  Enter: Select  q: Quit",
        Screen::Inbox => {
            "n: New  Enter: View  d: Delete  r: Refresh  u/s/a: Filter  e: Editor  m: Menu  Tab: Switch  q: Quit"
        }
        Screen::Drafts => "n: New  Enter: Resume  d: Delete  r: Refresh  m: Menu  Tab: Switch  q: Quit",
        Screen::Sent => "n: New  Enter: View  r: Refresh  e: Editor  m: Menu  Tab: Switch  q: Quit",
        Screen::Compose => match app.compose_step {
            ComposeStep::Preview => "Enter: Send  e: Edit  ESC: Save as draft  q: Quit",
            ComposeStep::NoEditor => "Any key: Return to menu  q: Quit",
            ComposeStep::Editing => "Editor is opening...",
        },
    };

    let status = app
        .status_message
        .as_ref()
        .map(|s| format!("  {}  ", s))
        .unwrap_or_default();

    let error = app
        .error_message
        .as_ref()
        .map(|e| format!("  {}  ", e))
        .unwrap_or_default();

    let footer_line = Line::from(vec![
        Span::styled(
            format!(" {} ", help_text),
            Style::default().add_modifier(Modifier::DIM),
        ),
        if !status.is_empty() {
            Span::styled(status, Style::default().fg(SUCCESS_COLOR))
        } else {
            Span::raw("")
        },
        if !error.is_empty() {
            Span::styled(error, Style::default().fg(ERROR_COLOR))
        } else {
            Span::raw("")
        },
    ]);

    let footer = Paragraph::new(footer_line);
    frame.render_widget(footer, area);
}

fn render_account_selection(frame: &mut Frame, app: &App, area: Rect) {
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
            Constraint::Length(6),
            Constraint::Length(list_height as u16),
            Constraint::Min(0),
        ])
        .split(horizontal_center[1]);

    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "VERO",
            Style::default()
                .fg(PRIMARY_COLOR)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Terminal Email Client",
            Style::default().fg(Color::Reset),
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
        .map(|(i, account)| {
            let is_selected = i == app.account_selected;

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

    let list = List::new(items);
    frame.render_widget(list, inner[1]);
}

fn render_inbox_screen(frame: &mut Frame, app: &App, area: Rect) {
    if app.inbox_view_mode == ViewMode::Detail {
        render_email_detail(frame, app, area, &app.inbox_emails, app.inbox_selected);
        return;
    }

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .margin(1)
        .split(area);

    let filter_str = match app.inbox_filter {
        crate::models::InboxFilter::Unseen => "Unseen",
        crate::models::InboxFilter::Seen => "Seen",
        crate::models::InboxFilter::All => "All",
    };

    let title = format!(" ▼ Inbox - {} ({}) ", filter_str, app.inbox_emails.len());

    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(title, Style::default().fg(PRIMARY_COLOR)));

    if app.inbox_loading {
        let loading_text = format!("{} Loading emails...", app.spinner_char());
        let loading = Paragraph::new(loading_text)
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(loading, inner[0]);
        return;
    }

    if let Some(ref error) = app.inbox_error {
        let error_text = Paragraph::new(error.as_str())
            .block(block)
            .style(Style::default().fg(ERROR_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(error_text, inner[0]);
        return;
    }

    if app.inbox_emails.is_empty() {
        let empty = Paragraph::new("No emails found")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(empty, inner[0]);
        return;
    }

    let available_width = inner[0].width.saturating_sub(4) as usize;

    let items: Vec<ListItem> = app
        .inbox_emails
        .iter()
        .enumerate()
        .map(|(i, email)| {
            let is_selected = i == app.inbox_selected;

            let (bg_color, fg_color, modifier) = if is_selected {
                (
                    PRIMARY_COLOR,
                    Color::Reset,
                    Modifier::BOLD | Modifier::REVERSED,
                )
            } else {
                (Color::Reset, Color::Reset, Modifier::empty())
            };

            let subject_max = available_width.saturating_sub(30).max(20);
            let from_max = 25;

            let subject = if email.subject.len() > subject_max {
                let truncated: String = email
                    .subject
                    .chars()
                    .take(subject_max.saturating_sub(1))
                    .collect();
                format!("{}…", truncated)
            } else {
                email.subject.clone()
            };

            let from = if email.from.len() > from_max {
                let truncated: String = email
                    .from
                    .chars()
                    .take(from_max.saturating_sub(1))
                    .collect();
                format!("{}…", truncated)
            } else {
                email.from.clone()
            };

            let padding = available_width.saturating_sub(subject.len() + from.len() + 2);
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
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, inner[0]);
}

fn render_drafts_screen(frame: &mut Frame, app: &App, area: Rect) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .margin(1)
        .split(area);

    let title = format!(" ✦ Drafts ({}) ", app.drafts.len());

    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(title, Style::default().fg(PRIMARY_COLOR)));

    if let Some(ref error) = app.drafts_error {
        let error_text = Paragraph::new(error.as_str())
            .block(block)
            .style(Style::default().fg(ERROR_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(error_text, inner[0]);
        return;
    }

    if app.drafts.is_empty() {
        let empty = Paragraph::new("No drafts found")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(empty, inner[0]);
        return;
    }

    let available_width = inner[0].width.saturating_sub(4) as usize;

    let items: Vec<ListItem> = app
        .drafts
        .iter()
        .enumerate()
        .map(|(i, (path, draft))| {
            let is_selected = i == app.drafts_selected;

            let (bg_color, fg_color, modifier) = if is_selected {
                (
                    PRIMARY_COLOR,
                    Color::Reset,
                    Modifier::BOLD | Modifier::REVERSED,
                )
            } else {
                (Color::Reset, Color::Reset, Modifier::empty())
            };

            let subject_max = available_width.saturating_sub(35).max(20);
            let to_max = 30;

            let subject_raw = if draft.subject.is_empty() {
                "(No Subject)".to_string()
            } else {
                draft.subject.clone()
            };

            let subject = if subject_raw.len() > subject_max {
                let truncated: String = subject_raw
                    .chars()
                    .take(subject_max.saturating_sub(1))
                    .collect();
                format!("{}…", truncated)
            } else {
                subject_raw
            };

            let to_raw = if draft.to.is_empty() {
                Path::new(path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            } else {
                format!("To: {}", draft.to)
            };

            let to_text = if to_raw.len() > to_max {
                let truncated: String = to_raw.chars().take(to_max.saturating_sub(1)).collect();
                format!("{}…", truncated)
            } else {
                to_raw
            };

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

    let list = List::new(items).block(block);
    frame.render_widget(list, inner[0]);
}

fn render_sent_screen(frame: &mut Frame, app: &App, area: Rect) {
    if app.sent_view_mode == ViewMode::Detail {
        render_email_detail(frame, app, area, &app.sent_emails, app.sent_selected);
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

    if app.sent_loading {
        let loading_text = format!("{} Loading sent emails...", app.spinner_char());
        let loading = Paragraph::new(loading_text)
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(loading, inner[0]);
        return;
    }

    if let Some(ref error) = app.sent_error {
        let error_text = Paragraph::new(error.as_str())
            .block(block)
            .style(Style::default().fg(ERROR_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(error_text, inner[0]);
        return;
    }

    if app.sent_emails.is_empty() {
        let empty = Paragraph::new("No sent emails found")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(empty, inner[0]);
        return;
    }

    let available_width = inner[0].width.saturating_sub(4) as usize;

    let items: Vec<ListItem> = app
        .sent_emails
        .iter()
        .enumerate()
        .map(|(i, email)| {
            let is_selected = i == app.sent_selected;

            let (bg_color, fg_color, modifier) = if is_selected {
                (
                    PRIMARY_COLOR,
                    Color::Reset,
                    Modifier::BOLD | Modifier::REVERSED,
                )
            } else {
                (Color::Reset, Color::Reset, Modifier::empty())
            };

            let unknown = String::from("Unknown");
            let to = email.to.as_ref().unwrap_or(&unknown);

            let subject_max = available_width.saturating_sub(35).max(20);
            let to_max = 30;

            let subject = if email.subject.len() > subject_max {
                let truncated: String = email
                    .subject
                    .chars()
                    .take(subject_max.saturating_sub(1))
                    .collect();
                format!("{}…", truncated)
            } else {
                email.subject.clone()
            };

            let to_text = if to.len() > to_max {
                let truncated: String = to.chars().take(to_max.saturating_sub(5)).collect();
                format!("To: {}…", truncated)
            } else {
                format!("To: {}", to)
            };

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

    let list = List::new(items).block(block);
    frame.render_widget(list, inner[0]);
}

fn render_email_detail(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    emails: &[crate::models::Email],
    selected: usize,
) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .margin(1)
        .split(area);

    if selected >= emails.len() {
        return;
    }

    let email = &emails[selected];

    let header_block = Block::default().borders(Borders::NONE).title(Span::styled(
        format!(" {} ", email.subject),
        Style::default().fg(PRIMARY_COLOR),
    ));

    let unknown = String::from("Unknown");
    let to = email.to.as_ref().unwrap_or(&unknown);

    let header_text = vec![
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

    let header = Paragraph::new(header_text).block(header_block);
    frame.render_widget(header, inner[0]);

    let scroll_offset = if app.screen == crate::tui::app::Screen::Inbox {
        app.inbox_scroll_offset
    } else {
        app.sent_scroll_offset
    };

    let body_block = Block::default().borders(Borders::NONE).title(Span::styled(
        "Body (j/k to scroll)".to_string(),
        Style::default().add_modifier(Modifier::DIM),
    ));

    let body = Paragraph::new(sanitize_email_body(email.body.as_str()))
        .block(body_block)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));
    frame.render_widget(body, inner[1]);
}

fn sanitize_email_body(body: &str) -> String {
    let mut sanitized = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\r' if matches!(chars.peek(), Some('\n')) => {}
            '\r' => sanitized.push('\n'),
            '\t' => sanitized.push_str("    "),
            '\n' => sanitized.push('\n'),
            _ if ch.is_control() => {}
            _ => sanitized.push(ch),
        }
    }

    sanitized
}

fn render_compose_screen(frame: &mut Frame, app: &App, area: Rect) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .margin(1)
        .split(area);

    match app.compose_step {
        ComposeStep::NoEditor => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ERROR_COLOR))
                .title(Span::styled(
                    " ◇ Compose - No Editor Configured ",
                    Style::default().fg(ERROR_COLOR),
                ));

            let error_text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No editor configured!",
                    Style::default()
                        .fg(ERROR_COLOR)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "To compose emails, add an editor to ~/.vero.yml:",
                    Style::default(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  editor: vim",
                    Style::default().add_modifier(Modifier::DIM),
                )),
                Line::from(Span::styled(
                    "  editor: nano",
                    Style::default().add_modifier(Modifier::DIM),
                )),
                Line::from(Span::styled(
                    "  editor: \"nvim -c 'startinsert'\"",
                    Style::default().add_modifier(Modifier::DIM),
                )),
                Line::from(Span::styled(
                    "  editor: emacs",
                    Style::default().add_modifier(Modifier::DIM),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press any key to return to menu.",
                    Style::default()
                        .fg(SUCCESS_COLOR)
                        .add_modifier(Modifier::ITALIC),
                )),
            ];

            let error_paragraph = Paragraph::new(error_text)
                .block(block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(error_paragraph, inner[0]);
        }
        ComposeStep::Editing => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PRIMARY_COLOR))
                .title(Span::styled(
                    " ◇ Compose - Opening Editor ",
                    Style::default().fg(PRIMARY_COLOR),
                ));

            let text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Opening editor...",
                    Style::default().add_modifier(Modifier::ITALIC),
                )),
            ];

            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(paragraph, inner[0]);
        }
        ComposeStep::Preview => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PRIMARY_COLOR))
                .title(Span::styled(
                    " ◇ Compose - Preview ",
                    Style::default().fg(PRIMARY_COLOR),
                ));

            let mut preview_lines = vec![Line::from(vec![
                Span::styled(
                    "To: ",
                    Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
                ),
                Span::styled(&app.compose_draft.to, Style::default()),
            ])];

            if !app.compose_draft.cc.is_empty() {
                preview_lines.push(Line::from(vec![
                    Span::styled(
                        "CC: ",
                        Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
                    ),
                    Span::styled(&app.compose_draft.cc, Style::default()),
                ]));
            }

            if !app.compose_draft.bcc.is_empty() {
                preview_lines.push(Line::from(vec![
                    Span::styled(
                        "BCC: ",
                        Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
                    ),
                    Span::styled(&app.compose_draft.bcc, Style::default()),
                ]));
            }

            preview_lines.push(Line::from(vec![
                Span::styled(
                    "Subject: ",
                    Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
                ),
                Span::styled(&app.compose_draft.subject, Style::default()),
            ]));

            if !app.compose_draft.attachments.is_empty() {
                preview_lines.push(Line::from(vec![
                    Span::styled(
                        "Attachments: ",
                        Style::default().add_modifier(Modifier::BOLD | Modifier::DIM),
                    ),
                    Span::styled(
                        format!("{} file(s)", app.compose_draft.attachments.len()),
                        Style::default(),
                    ),
                ]));
            }

            preview_lines.push(Line::from(""));
            preview_lines.push(Line::from(app.compose_draft.body.as_str()));
            preview_lines.push(Line::from(""));
            preview_lines.push(Line::from(Span::styled(
                "Enter: Send  |  e: Edit again  |  ESC: Cancel",
                Style::default()
                    .fg(SUCCESS_COLOR)
                    .add_modifier(Modifier::ITALIC),
            )));

            let preview = Paragraph::new(preview_lines)
                .block(block)
                .wrap(Wrap { trim: false });
            frame.render_widget(preview, inner[0]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_email_body;

    #[test]
    fn sanitize_email_body_normalizes_crlf_and_tabs() {
        let body = "one\r\ntwo\rthree\tfour";

        assert_eq!(sanitize_email_body(body), "one\ntwo\nthree    four");
    }

    #[test]
    fn sanitize_email_body_drops_other_control_chars() {
        let body = "he\x00llo\x1b!";

        assert_eq!(sanitize_email_body(body), "hello!");
    }
}
