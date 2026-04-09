use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::storage;
use crate::tui::app::{FocusedElement, Screen};
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    if app.focused == FocusedElement::MenuBar {
        handle_menu_focus(app, key);
        return Ok(());
    }

    match key.code {
        KeyCode::Char('n') => {
            app.navigate_to(Screen::Compose);
        }
        KeyCode::Enter => {
            if let Some((path, _)) = app.drafts.get(app.drafts_selected).cloned() {
                app.resume_draft(path);
            }
        }
        KeyCode::Char('d') => {
            if let Some((path, _)) = app.drafts.get(app.drafts_selected).cloned() {
                storage::delete_draft_file(&path).ok();
                app.drafts.remove(app.drafts_selected);
                if app.drafts_selected > 0 && app.drafts_selected >= app.drafts.len() {
                    app.drafts_selected -= 1;
                }
                app.status_message = Some("Draft deleted.".to_string());
            }
        }
        KeyCode::Char('r') => {
            app.needs_drafts_load = true;
        }
        KeyCode::Esc => app.navigate_to(Screen::Inbox),
        KeyCode::Char('m') => app.toggle_focus(),
        KeyCode::Tab => app.tab_next_screen(),
        KeyCode::BackTab => app.tab_prev_screen(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
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
        KeyCode::Char('m') => app.toggle_focus(),
        KeyCode::Tab => app.tab_next_screen(),
        KeyCode::BackTab => app.tab_prev_screen(),
        _ => {}
    }
}
