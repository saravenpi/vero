use crate::storage;
use crate::tui::App;

pub(in crate::tui::runtime) fn handle_drafts_load(app: &mut App) {
    if !app.needs_drafts_load {
        return;
    }

    app.needs_drafts_load = false;

    if let Some(account) = app.current_account.as_ref() {
        match storage::load_drafts(&account.email) {
            Ok(drafts) => {
                app.drafts = drafts;
                app.clamp_drafts_selection();
                app.drafts_error = None;
            }
            Err(e) => {
                app.set_drafts_error(format!("Failed to load drafts: {}", e));
            }
        }
    }
}
