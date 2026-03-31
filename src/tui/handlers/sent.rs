use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::models::ViewMode;
use crate::tui::app::{FocusedElement, Screen};
use crate::tui::external::open_email_in_viewer;
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    if app.focused == FocusedElement::MenuBar {
        handle_menu_focus(app, key);
        return Ok(());
    }

    if app.sent_view_mode == ViewMode::Detail {
        handle_detail_view(app, key)?;
        return Ok(());
    }

    match key.code {
        KeyCode::Esc => app.navigate_to(Screen::Inbox),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Enter => {
            if app.sent_selected < app.sent_emails.len() {
                app.sent_scroll_offset = 0;
                app.sent_view_mode = ViewMode::Detail;
            }
        }
        KeyCode::Char('r') => {
            app.cancel_sent_load = false;
            app.needs_sent_load = true;
            app.sent_loading = true;
            app.sent_error = None;
        }
        KeyCode::Tab | KeyCode::BackTab => app.toggle_focus(),
        _ => {}
    }

    Ok(())
}

fn handle_menu_focus(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            if app.config.accounts.len() > 1 {
                app.navigate_to(Screen::AccountSelection);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => app.menu_next(),
        KeyCode::Up | KeyCode::Char('k') => app.menu_previous(),
        KeyCode::Enter => app.menu_select(),
        KeyCode::Tab | KeyCode::BackTab => app.toggle_focus(),
        _ => {}
    }
}

fn handle_detail_view(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.sent_view_mode = ViewMode::List;
            app.sent_scroll_offset = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.sent_scroll_offset = app.sent_scroll_offset.saturating_add(1);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.sent_scroll_offset = app.sent_scroll_offset.saturating_sub(1);
        }
        KeyCode::PageDown => {
            app.sent_scroll_offset = app.sent_scroll_offset.saturating_add(10);
        }
        KeyCode::PageUp => {
            app.sent_scroll_offset = app.sent_scroll_offset.saturating_sub(10);
        }
        KeyCode::Tab | KeyCode::BackTab => app.toggle_focus(),
        KeyCode::Char('e') => {
            if app.sent_selected < app.sent_emails.len() {
                let viewer = app.config.editor.as_ref().or(app.config.viewer.as_ref());
                if let Some(viewer) = viewer {
                    let email = &app.sent_emails[app.sent_selected];
                    open_email_in_viewer(viewer, email)?;
                    app.needs_full_redraw = true;
                }
            }
        }
        _ => {}
    }

    Ok(())
}
