use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::Screen;
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Enter => {
            if app.account_selected < app.config.accounts.len() {
                app.current_account = Some(app.config.accounts[app.account_selected].clone());
                app.navigate_to(Screen::Inbox);
            }
        }
        _ => {}
    }

    Ok(())
}
