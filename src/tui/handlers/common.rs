use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::Screen;
use crate::tui::App;

pub(crate) fn handle_menu_focus(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.navigate_to(Screen::AccountSelection);
        }
        KeyCode::Down | KeyCode::Char('j') => app.menu_next(),
        KeyCode::Up | KeyCode::Char('k') => app.menu_previous(),
        KeyCode::Enter => app.menu_select(),
        KeyCode::Tab => app.tab_next_screen(),
        KeyCode::BackTab => app.tab_prev_screen(),
        _ => {}
    }
}

pub(crate) fn handle_list_jump(app: &mut App, key: KeyEvent) -> bool {
    app.handle_list_jump_key(key.code)
}
