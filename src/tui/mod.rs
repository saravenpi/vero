pub mod app;
mod events;
mod ui;

pub use app::App;
pub use events::{AppEvent, EventHandler};
pub use ui::render;

use crossterm::event::KeyEvent;

pub fn is_quit_key(key: &KeyEvent) -> bool {
    matches!(key.code, crossterm::event::KeyCode::Char('q'))
}
