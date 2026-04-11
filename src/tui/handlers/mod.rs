mod account_selection;
mod common;
mod compose;
mod drafts;
mod inbox;
mod sent;
mod signatures;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::Screen;
use crate::tui::App;

pub async fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    if matches!(key.code, KeyCode::Char('n'))
        && !app.is_list_search_editing()
        && matches!(app.screen, Screen::Inbox | Screen::Drafts | Screen::Sent)
    {
        app.navigate_to(Screen::Compose);
        return Ok(());
    }

    match app.screen {
        Screen::AccountSelection => account_selection::handle(app, key).await,
        Screen::Inbox => inbox::handle(app, key).await,
        Screen::Drafts => drafts::handle(app, key).await,
        Screen::Sent => sent::handle(app, key).await,
        Screen::Compose => compose::handle(app, key).await,
        Screen::Signatures => signatures::handle(app, key).await,
    }
}
