use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::models::{InboxFilter, ViewMode};
use crate::services;
use crate::tui::app::{FocusedElement, Screen};
use crate::tui::external::open_email_in_viewer;
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    if app.focused == FocusedElement::MenuBar {
        handle_menu_focus(app, key);
        return Ok(());
    }

    if app.inbox_view_mode == ViewMode::Detail {
        handle_detail_view(app, key)?;
        return Ok(());
    }

    match key.code {
        KeyCode::Esc => app.navigate_to(Screen::Inbox),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Enter => {
            if app.inbox_selected < app.inbox_emails.len() {
                let account = app.current_account.as_ref().unwrap().clone();
                let email = app.inbox_emails[app.inbox_selected].clone();
                let email = services::read_loaded_inbox_email(&account, email).await?;

                app.inbox_emails[app.inbox_selected] = email;
                app.inbox_scroll_offset = 0;
                app.inbox_view_mode = ViewMode::Detail;
            }
        }
        KeyCode::Char('d') => {
            if app.inbox_selected < app.inbox_emails.len() {
                let account = app.current_account.as_ref().unwrap().clone();
                let email = app.inbox_emails[app.inbox_selected].clone();

                services::delete_loaded_inbox_email(&account, &email).await?;

                app.inbox_emails.remove(app.inbox_selected);
                if app.inbox_selected > 0 && app.inbox_selected >= app.inbox_emails.len() {
                    app.inbox_selected -= 1;
                }

                app.status_message = Some("Email deleted".to_string());
            }
        }
        KeyCode::Char('u') => set_filter(app, InboxFilter::Unseen),
        KeyCode::Char('s') => set_filter(app, InboxFilter::Seen),
        KeyCode::Char('a') => set_filter(app, InboxFilter::All),
        KeyCode::Char('r') => {
            app.cancel_inbox_load = false;
            app.needs_inbox_load = true;
            app.inbox_loading = true;
            app.inbox_error = None;
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
            app.inbox_view_mode = ViewMode::List;
            app.inbox_scroll_offset = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.inbox_scroll_offset = app.inbox_scroll_offset.saturating_add(1);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.inbox_scroll_offset = app.inbox_scroll_offset.saturating_sub(1);
        }
        KeyCode::PageDown => {
            app.inbox_scroll_offset = app.inbox_scroll_offset.saturating_add(10);
        }
        KeyCode::PageUp => {
            app.inbox_scroll_offset = app.inbox_scroll_offset.saturating_sub(10);
        }
        KeyCode::Tab | KeyCode::BackTab => app.toggle_focus(),
        KeyCode::Char('e') => {
            if app.inbox_selected < app.inbox_emails.len() {
                let viewer = app.config.editor.as_ref().or(app.config.viewer.as_ref());
                if let Some(viewer) = viewer {
                    let email = &app.inbox_emails[app.inbox_selected];
                    open_email_in_viewer(viewer, email)?;
                    app.needs_full_redraw = true;
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn set_filter(app: &mut App, filter: InboxFilter) {
    app.inbox_filter = filter;
    app.needs_inbox_load = true;
    app.inbox_loading = true;
    app.inbox_error = None;
}
