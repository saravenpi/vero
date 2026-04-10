mod detail;
mod list;

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::models::ViewMode;
use crate::tui::app::FocusedElement;
use crate::tui::handlers::common;
use crate::tui::App;

#[cfg(test)]
mod tests;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    if app.focused == FocusedElement::MenuBar {
        common::handle_menu_focus(app, key);
        return Ok(());
    }

    if app.inbox_view_mode == ViewMode::Detail {
        detail::handle(app, key)?;
        return Ok(());
    }

    if common::handle_list_jump(app, key) {
        return Ok(());
    }

    list::handle(app, key).await
}
