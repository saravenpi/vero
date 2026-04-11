use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

use crate::models::ViewMode;
use crate::tui::app::{App, ComposeStep, Screen};
use crate::tui::ui::theme::{muted_text_style, ERROR_COLOR, PRIMARY_COLOR, SUCCESS_COLOR};

pub(crate) fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = help_text(app);

    let help_line = Line::from(Span::styled(format!(" {} ", help_text), muted_text_style()));

    let status_line = if let Some(prompt) = search_prompt(app) {
        Line::from(Span::styled(
            format!(" {} ", prompt),
            Style::default().fg(PRIMARY_COLOR),
        ))
    } else if let Some(msg) = &app.error_message {
        Line::from(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(ERROR_COLOR),
        ))
    } else if let Some(msg) = screen_error(app) {
        Line::from(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(ERROR_COLOR),
        ))
    } else if let Some(msg) = &app.status_message {
        Line::from(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(SUCCESS_COLOR),
        ))
    } else if app.inbox_open_loading {
        Line::from(Span::styled(
            format!(" {} Opening email… ", app.spinner_char()),
            muted_text_style(),
        ))
    } else if app.is_downloading_attachment {
        Line::from(Span::styled(
            format!(" {} Downloading… ", app.spinner_char()),
            Style::default().fg(SUCCESS_COLOR),
        ))
    } else if app.is_sending_email {
        Line::from(Span::styled(
            format!(" {} Sending email… ", app.spinner_char()),
            Style::default().fg(SUCCESS_COLOR),
        ))
    } else if app.inbox_loading {
        Line::from(Span::styled(
            format!(" {} Refreshing inbox… ", app.spinner_char()),
            muted_text_style(),
        ))
    } else if app.sent_loading {
        Line::from(Span::styled(
            format!(" {} Refreshing sent… ", app.spinner_char()),
            muted_text_style(),
        ))
    } else {
        Line::from("")
    };

    frame.render_widget(
        Paragraph::new(Text::from(vec![help_line, status_line])),
        area,
    );
}

fn screen_error(app: &App) -> Option<&str> {
    match app.screen {
        Screen::Inbox => app.inbox_error.as_deref(),
        Screen::Sent => app.sent_error.as_deref(),
        Screen::Drafts => app.drafts_error.as_deref(),
        _ => None,
    }
}

fn help_text(app: &App) -> String {
    if app.is_list_search_editing() {
        return "Type to filter  Enter: Keep  Esc: Done".to_string();
    }

    match app.screen {
        Screen::AccountSelection => "↑/↓: Navigate  Enter: Select".to_string(),
        Screen::Inbox => match app.inbox_view_mode {
            ViewMode::List if app.inbox_search().is_active() => {
                "n: New  d: Delete  /: Edit  Esc: Clear search  Tab: Switch".to_string()
            }
            ViewMode::List => "n: New  d: Delete  /: Search  r: Refresh  Tab: Switch".to_string(),
            ViewMode::Detail if app.inbox_show_attachments => {
                "j/k: Navigate  Enter: Download  a: All  Esc: Back".to_string()
            }
            ViewMode::Detail => {
                let has_attachments = app
                    .selected_inbox_email()
                    .is_some_and(|email| !email.attachments.is_empty());
                let quotes_hint = if app.inbox_collapse_quotes {
                    "z: Show history"
                } else {
                    "z: Hide history"
                };
                if has_attachments {
                    format!("j/k: Scroll  {quotes_hint}  d: Attachments  e: Editor  Esc: Back")
                } else {
                    format!("j/k: Scroll  {quotes_hint}  e: Editor  Esc: Back  Tab: Switch")
                }
            }
        },
        Screen::Drafts if app.drafts_search().is_active() => {
            "n: New  /: Edit  Esc: Clear search  Tab: Switch".to_string()
        }
        Screen::Drafts => "n: New  /: Search  d: Delete  Tab: Switch".to_string(),
        Screen::Sent => match app.sent_view_mode {
            ViewMode::List if app.sent_search().is_active() => {
                "n: New  d: Delete  /: Edit  Esc: Clear search  Tab: Switch".to_string()
            }
            ViewMode::List => "n: New  d: Delete  /: Search  r: Refresh  Tab: Switch".to_string(),
            ViewMode::Detail => "j/k: Scroll  e: Editor  Esc: Back  Tab: Switch".to_string(),
        },
        Screen::Compose => match app.compose_step {
            ComposeStep::Preview => {
                "j/k: Scroll  Enter: Send  e: Edit  Esc: Save as draft".to_string()
            }
            ComposeStep::NoEditor => "Any key: Return to menu".to_string(),
            ComposeStep::Editing => "Editor is opening...".to_string(),
        },
        Screen::Signatures => "e: Edit  Tab: Switch".to_string(),
    }
}

fn search_prompt(app: &App) -> Option<String> {
    if !app.is_list_search_editing() {
        return None;
    }

    let query = app.current_list_search()?.display_query();
    if query.is_empty() {
        return Some(match app.screen {
            Screen::Inbox => "/ search subject or sender".to_string(),
            Screen::Drafts => "/ search subject or recipient".to_string(),
            Screen::Sent => "/ search subject or contact".to_string(),
            _ => "/ search".to_string(),
        });
    }

    let matches = app.current_list_search_match_count().unwrap_or(0);
    let suffix = if matches == 1 { "" } else { "es" };
    Some(format!("/{query}  {matches} match{suffix}"))
}
