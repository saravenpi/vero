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

    if app.handle_list_search_key(key) {
        return Ok(());
    }

    if matches!(key.code, KeyCode::Char('/')) && app.begin_list_search() {
        return Ok(());
    }

    if matches!(key.code, KeyCode::Esc) && app.clear_current_list_search() {
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
            if app.selected_sent_email().is_some() {
                app.sent_scroll_offset = 0;
                app.sent_view_mode = ViewMode::Detail;
                app.needs_full_redraw = true;
            }
        }
        KeyCode::Char('d') => delete_selected_sent_email(app)?,
        KeyCode::Char('r') => {
            app.needs_sent_load = true;
            app.sent_loading = true;
        }
        KeyCode::Tab => app.tab_next_screen(),
        KeyCode::BackTab => app.tab_prev_screen(),
        _ => {}
    }

    Ok(())
}

fn delete_selected_sent_email(app: &mut App) -> Result<()> {
    let Some(account) = app.current_account.clone() else {
        app.set_error("No account selected");
        return Ok(());
    };

    let Some(index) = app.selected_sent_index() else {
        return Ok(());
    };

    let email = &app.sent_emails[index];
    storage::delete_sent_email(&account.email, email)?;

    app.sent_emails.remove(index);
    app.clamp_sent_selection();
    app.set_status("Email deleted.");

    Ok(())
}

fn handle_detail_view(app: &mut App, key: KeyEvent) -> Result<()> {
    if common::handle_list_jump(app, key) {
        return Ok(());
    }

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
            if let Some(email) = app.selected_sent_email() {
                if let Some(viewer) = app.config.viewer_command() {
                    open_email_in_viewer(&viewer, email)?;
                    app.needs_full_redraw = true;
                } else {
                    app.set_error("No viewer configured in ~/.vero.yml and $EDITOR is unset");
                }
            }
        }
        _ => {}
    }

    Ok(())
}
