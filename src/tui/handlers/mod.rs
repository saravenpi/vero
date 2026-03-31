mod account_selection;
mod compose;
mod inbox;
mod sent;

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::tui::app::Screen;
use crate::tui::App;

pub async fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.screen {
        Screen::AccountSelection => account_selection::handle(app, key).await,
        Screen::Inbox => inbox::handle(app, key).await,
        Screen::Sent => sent::handle(app, key).await,
        Screen::Compose => compose::handle(app, key).await,
    }
}
