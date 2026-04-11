use anyhow::Result;

use crate::services;
use crate::storage;
use crate::tui::app::{ComposeStep, Screen};
use crate::tui::App;

pub(super) fn handle_editor_open(app: &mut App) -> Result<()> {
    if !app.needs_editor_open {
        return Ok(());
    }

    app.needs_editor_open = false;

    let Some(editor) = app.config.editor_command() else {
        app.compose_step = ComposeStep::NoEditor;
        return Ok(());
    };

    let Some(account) = app.current_account.as_ref() else {
        app.set_error("No account selected");
        app.navigate_to(Screen::Inbox);
        return Ok(());
    };

    let existing_draft = app.compose_draft_path.clone();
    let is_existing = existing_draft.is_some();

    let draft_path = if let Some(path) = existing_draft {
        path
    } else {
        match storage::create_draft_file(&account.email) {
            Ok(path) => {
                app.compose_draft_path = Some(path.clone());
                path
            }
            Err(error) => {
                app.set_error(format!("Failed to create draft: {}", error));
                app.navigate_to(Screen::Inbox);
                return Ok(());
            }
        }
    };

    match crate::tui::external::open_editor_for_draft(&editor, &draft_path) {
        Ok(()) => {
            app.needs_full_redraw = true;
            match services::parse_draft_input(&draft_path) {
                Ok(parsed) => {
                    app.compose_draft = parsed.to_draft();
                    app.compose_step = ComposeStep::Preview;
                    app.error_message = None;
                    app.status_ttl = 0;
                }
                Err(error) => {
                    app.set_error(format!("Draft parsing error: {}", error));
                    if !is_existing {
                        storage::delete_draft_file(&draft_path).ok();
                    }
                    app.navigate_to(Screen::Drafts);
                }
            }
        }
        Err(error) => {
            app.needs_full_redraw = true;
            app.set_error(format!("Editor error: {}", error));
            if !is_existing {
                storage::delete_draft_file(&draft_path).ok();
            }
            app.navigate_to(Screen::Drafts);
        }
    }

    Ok(())
}
