use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::services;
use crate::tui::app::{ComposeStep, Screen};
use crate::tui::external::open_editor_for_draft;
use crate::tui::handlers::common;
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.compose_step {
        ComposeStep::NoEditor => {
            app.navigate_to(Screen::Inbox);
        }
        ComposeStep::Editing => {}
        ComposeStep::Preview => {
            if common::handle_list_jump(app, key) {
                return Ok(());
            }

            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    app.compose_preview_scroll_offset =
                        app.compose_preview_scroll_offset.saturating_add(1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.compose_preview_scroll_offset =
                        app.compose_preview_scroll_offset.saturating_sub(1);
                }
                KeyCode::PageDown => {
                    app.compose_preview_scroll_offset =
                        app.compose_preview_scroll_offset.saturating_add(10);
                }
                KeyCode::PageUp => {
                    app.compose_preview_scroll_offset =
                        app.compose_preview_scroll_offset.saturating_sub(10);
                }
                KeyCode::Enter => {
                    if !app.needs_email_send {
                        app.needs_email_send = true;
                    }
                }
                KeyCode::Char('e') => {
                    if let Some(draft_path) = app.compose_draft_path.as_ref() {
                        if let Some(editor) = app.config.editor_command() {
                            match open_editor_for_draft(&editor, draft_path) {
                                Ok(()) => {
                                    app.needs_full_redraw = true;
                                    match services::parse_draft_input(draft_path) {
                                        Ok(parsed) => {
                                            app.compose_draft = parsed.to_draft();
                                            app.compose_preview_scroll_offset = 0;
                                            app.error_message = None;
                                            app.status_ttl = 0;
                                        }
                                        Err(error) => {
                                            app.set_error(format!(
                                                "Draft parsing error: {}",
                                                error
                                            ));
                                        }
                                    }
                                }
                                Err(error) => {
                                    app.needs_full_redraw = true;
                                    app.set_error(format!("Editor error: {}", error));
                                }
                            }
                        } else {
                            app.set_error(
                                "No editor configured in ~/.vero.yml and $EDITOR is unset",
                            );
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
            }
        }
    }

    Ok(())
}
