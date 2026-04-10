use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::models::ViewMode;
use crate::storage;
use crate::tui::app::{FocusedElement, Screen};
use crate::tui::external::open_email_in_viewer;
use crate::tui::handlers::common;
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    if app.focused == FocusedElement::MenuBar {
        common::handle_menu_focus(app, key);
        return Ok(());
    }

    if app.sent_view_mode == ViewMode::Detail {
        handle_detail_view(app, key)?;
        return Ok(());
    }

    if common::handle_list_jump(app, key) {
        return Ok(());
    }

    match key.code {
        KeyCode::Esc => app.navigate_to(Screen::AccountSelection),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Enter => {
            if app.sent_selected < app.sent_emails.len() {
                app.sent_scroll_offset = 0;
                app.sent_view_mode = ViewMode::Detail;
                app.needs_full_redraw = true;
            }
        }
        KeyCode::Char('d') => delete_selected_sent_email(app)?,
        KeyCode::Char('r') => {
            app.needs_sent_load = true;
            app.sent_loading = true;
            app.sent_error = None;
        }
        KeyCode::Tab => app.tab_next_screen(),
        KeyCode::BackTab => app.tab_prev_screen(),
        _ => {}
    }

    Ok(())
}

fn delete_selected_sent_email(app: &mut App) -> Result<()> {
    if app.sent_selected >= app.sent_emails.len() {
        return Ok(());
    }

    let Some(account) = app.current_account.clone() else {
        app.set_error("No account selected");
        return Ok(());
    };

    let email = &app.sent_emails[app.sent_selected];
    storage::delete_sent_email(&account.email, email)?;

    app.sent_emails.remove(app.sent_selected);
    app.clamp_sent_selection();
    app.set_status("Email deleted.");

    Ok(())
}

fn handle_detail_view(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.sent_view_mode = ViewMode::List;
            app.sent_scroll_offset = 0;
            app.needs_full_redraw = true;
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
        KeyCode::Tab => app.tab_next_screen(),
        KeyCode::BackTab => app.tab_prev_screen(),
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
