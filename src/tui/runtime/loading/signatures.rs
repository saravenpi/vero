use crate::storage;
use crate::tui::App;

pub fn handle_signature_load(app: &mut App) {
    if !app.needs_signature_load {
        return;
    }
    app.needs_signature_load = false;

    let Some(account) = app.current_account.as_ref() else {
        return;
    };

    app.signature_content = storage::load_signature(&account.email);
}
