use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::models::ViewMode;
use crate::tui::external::open_email_in_viewer;
use crate::tui::handlers::common;
use crate::tui::App;

pub(super) fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    if common::handle_list_jump(app, key) {
        return Ok(());
    }

    match key.code {
        KeyCode::Esc => {
            app.inbox_view_mode = ViewMode::List;
            app.inbox_scroll_offset = 0;
            if app.inbox_cache_loaded {
                app.refresh_inbox_emails(None);
            }
            app.needs_inbox_load = true;
            app.inbox_loading = true;
            app.inbox_error = None;
            app.needs_full_redraw = true;
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
        KeyCode::Tab => app.tab_next_screen(),
        KeyCode::BackTab => app.tab_prev_screen(),
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
