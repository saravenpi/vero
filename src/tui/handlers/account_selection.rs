use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::models::InboxFilter;
use crate::tui::app::Screen;
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Enter => {
            if app.account_selected < app.config.accounts.len() {
                app.current_account = Some(app.config.accounts[app.account_selected].clone());
                app.inbox_filter = InboxFilter::All;
                app.inbox_cached_emails.clear();
                app.inbox_cache_loaded = false;
                app.inbox_emails.clear();
                app.inbox_unseen_count = 0;
                app.navigate_to(Screen::Inbox);
            }
        }
        _ => {}
    }

    Ok(())
}
