use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::storage;
use crate::tui::app::{FocusedElement, Screen};
use crate::tui::handlers::common;
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    if app.focused == FocusedElement::MenuBar {
        common::handle_menu_focus(app, key);
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
        KeyCode::Char('n') => {
            app.navigate_to(Screen::Compose);
        }
        KeyCode::Enter => {
            if let Some((path, _)) = app.selected_draft().cloned() {
                app.resume_draft(path);
            }
        }
        KeyCode::Char('d') => {
            if let Some(index) = app.selected_draft_index() {
                let path = app.drafts[index].0.clone();
                storage::delete_draft_file(&path).ok();
                app.drafts.remove(index);
                app.clamp_drafts_selection();
                app.set_status("Draft deleted.");
            }
        }
        KeyCode::Char('r') => {
            app.needs_drafts_load = true;
        }
        KeyCode::Esc => app.navigate_to(Screen::AccountSelection),
        KeyCode::Tab => app.tab_next_screen(),
        KeyCode::BackTab => app.tab_prev_screen(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        _ => {}
    }

    Ok(())
}
