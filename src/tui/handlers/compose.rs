use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::services;
use crate::storage;
use crate::tui::app::{ComposeStep, Screen};
use crate::tui::external::open_editor_for_draft;
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.compose_step {
        ComposeStep::NoEditor => {
            app.navigate_to(Screen::Inbox);
        }
        ComposeStep::Editing => {}
        ComposeStep::Preview => match key.code {
            KeyCode::Enter => {
                let Some(account) = app.current_account.clone() else {
                    app.set_error("No account selected");
                    return Ok(());
                };
                services::send_draft(&account, app.compose_draft.clone()).await?;

                if let Some(draft_path) = app.compose_draft_path.as_ref() {
                    storage::delete_draft_file(draft_path).ok();
                }

                app.set_status("Email sent successfully!");
                app.navigate_to(Screen::Inbox);
            }
            KeyCode::Char('e') => {
                if let (Some(draft_path), Some(editor)) =
                    (app.compose_draft_path.as_ref(), app.config.editor.as_ref())
                {
                    match open_editor_for_draft(editor, draft_path) {
                        Ok(()) => {
                            app.needs_full_redraw = true;
                            match services::parse_draft_input(draft_path) {
                                Ok(parsed) => {
                                    app.compose_draft = parsed.to_draft();
                                    app.error_message = None;
                                    app.status_ttl = 0;
                                }
                                Err(error) => {
                                    app.set_error(format!("Draft parsing error: {}", error));
                                }
                            }
                        }
                        Err(error) => {
                            app.needs_full_redraw = true;
                            app.set_error(format!("Editor error: {}", error));
                        }
                    }
                }
            }
            KeyCode::Esc => {
                app.set_status("Draft saved.");
                app.navigate_to(Screen::Drafts);
            }
            KeyCode::Tab => app.tab_next_screen(),
            KeyCode::BackTab => app.tab_prev_screen(),
            _ => {}
        },
    }

    Ok(())
}
