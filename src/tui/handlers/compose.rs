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
                let account = app.current_account.clone().unwrap();
                services::send_draft(&account, app.compose_draft.clone()).await?;

                if let Some(draft_path) = app.compose_draft_path.as_ref() {
                    storage::delete_draft_file(draft_path).ok();
                }

                app.status_message = Some("Email sent successfully!".to_string());
                app.navigate_to(Screen::Inbox);
            }
            KeyCode::Char('e') => {
                if let Some(draft_path) = app.compose_draft_path.as_ref() {
                    match open_editor_for_draft(app.config.editor.as_ref().unwrap(), draft_path) {
                        Ok(()) => {
                            app.needs_full_redraw = true;
                            match services::parse_draft_input(draft_path) {
                                Ok(parsed) => {
                                    app.compose_draft = parsed.to_draft();
                                    app.error_message = None;
                                }
                                Err(error) => {
                                    app.error_message =
                                        Some(format!("Draft parsing error: {}", error));
                                }
                            }
                        }
                        Err(error) => {
                            app.needs_full_redraw = true;
                            app.error_message = Some(format!("Editor error: {}", error));
                        }
                    }
                }
            }
            KeyCode::Esc => {
                if let Some(draft_path) = app.compose_draft_path.as_ref() {
                    storage::delete_draft_file(draft_path).ok();
                }
                app.navigate_to(Screen::Inbox);
            }
            _ => {}
        },
    }

    Ok(())
}
