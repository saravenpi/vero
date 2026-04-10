use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

use crate::models::ViewMode;
use crate::tui::app::{App, ComposeStep, Screen};
use crate::tui::ui::theme::{ERROR_COLOR, SUCCESS_COLOR};

pub(crate) fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.screen {
        Screen::AccountSelection => "↑/↓: Navigate  Enter: Select",
        Screen::Inbox => match app.inbox_view_mode {
            ViewMode::List => "n: New  d: Delete  r: Refresh  Tab: Switch",
            ViewMode::Detail => "e: Editor  Esc: Back  Tab: Switch",
        },
        Screen::Drafts => {
            "n: New  Enter: Resume  d: Delete  r: Refresh  Tab: Switch"
        }
        Screen::Sent => match app.sent_view_mode {
            ViewMode::List => "n: New  d: Delete  r: Refresh  Tab: Switch",
            ViewMode::Detail => "e: Editor  Esc: Back  Tab: Switch",
        },
        Screen::Compose => match app.compose_step {
            ComposeStep::Preview => "Enter: Send  e: Edit  ESC: Save as draft",
            ComposeStep::NoEditor => "Any key: Return to menu",
            ComposeStep::Editing => "Editor is opening...",
        },
        Screen::Signatures => "e: Edit  Tab: Switch",
    };

    let help_line = Line::from(Span::styled(
        format!(" {} ", help_text),
        Style::default().add_modifier(Modifier::DIM),
    ));

    let status_line = if let Some(msg) = &app.error_message {
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
    } else if app.inbox_loading {
        Line::from(Span::styled(
            format!(" {} Refreshing inbox… ", app.spinner_char()),
            Style::default().add_modifier(Modifier::DIM),
        ))
    } else if app.sent_loading {
        Line::from(Span::styled(
            format!(" {} Refreshing sent… ", app.spinner_char()),
            Style::default().add_modifier(Modifier::DIM),
        ))
    } else {
        Line::from("")
    };

    frame.render_widget(Paragraph::new(Text::from(vec![help_line, status_line])), area);
}

fn screen_error(app: &App) -> Option<&str> {
    match app.screen {
        Screen::Inbox => app.inbox_error.as_deref(),
        Screen::Sent => app.sent_error.as_deref(),
        Screen::Drafts => app.drafts_error.as_deref(),
        _ => None,
    }
}
