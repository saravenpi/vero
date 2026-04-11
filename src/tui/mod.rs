pub mod app;
mod error_messages;
mod events;
mod external;
mod handlers;
mod runtime;
mod ui;

#[cfg(test)]
pub(crate) mod test_support;

pub use app::App;
pub use events::{AppEvent, EventHandler};
pub use runtime::run;
pub use ui::render;

use crossterm::event::KeyEvent;

pub fn is_quit_key(key: &KeyEvent) -> bool {
    matches!(key.code, crossterm::event::KeyCode::Char('q'))
}
