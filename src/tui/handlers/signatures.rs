use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::storage;
use crate::tui::app::{FocusedElement, Screen};
use crate::tui::external::open_editor_for_draft;
use crate::tui::handlers::common;
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    if app.focused == FocusedElement::MenuBar {
        common::handle_menu_focus(app, key);
        return Ok(());
    }

    match key.code {
        KeyCode::Char('e') => {
            let Some(editor) = app.config.editor.clone() else {
                app.set_error("No editor configured in ~/.vero.yml");
                return Ok(());
            };
            let Some(account) = app.current_account.as_ref() else {
                return Ok(());
            };
            let sig_path = storage::get_or_create_signature_path(&account.email.clone())?;
            match open_editor_for_draft(&editor, &sig_path) {
                Ok(()) => {
                    app.needs_full_redraw = true;
                    app.needs_signature_load = true;
                    app.set_status("Signature updated.");
                }
                Err(error) => {
                    app.needs_full_redraw = true;
                    app.set_error(format!("Editor error: {}", error));
                }
            }
        }
        KeyCode::Esc => app.navigate_to(Screen::AccountSelection),
        KeyCode::Tab => app.tab_next_screen(),
        KeyCode::BackTab => app.tab_prev_screen(),
        _ => {}
    }

    Ok(())
}
