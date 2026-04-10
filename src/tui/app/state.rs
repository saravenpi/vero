use super::{App, ComposeStep, FocusedElement, Screen};
use crate::config::VeroConfig;
use crate::models::{EmailDraft, InboxFilter, ViewMode};

impl App {
    pub fn new(config: VeroConfig) -> Self {
        let current_account = if config.accounts.len() == 1 {
            Some(config.accounts[0].clone())
        } else {
            None
        };

        let screen = if current_account.is_some() {
            Screen::Inbox
        } else {
            Screen::AccountSelection
        };

        let inbox_filter = InboxFilter::All;
        let should_load_inbox = current_account.is_some();

        Self {
            config,
            current_account,
            screen,
            focused: FocusedElement::MenuBar,
            account_selected: 0,
            menu_selected: 0,
            inbox_filter,
            inbox_cached_emails: Vec::new(),
            inbox_cache_loaded: false,
            inbox_emails: Vec::new(),
            inbox_selected: 0,
            inbox_view_mode: ViewMode::List,
            inbox_loading: should_load_inbox,
            inbox_error: None,
            inbox_unseen_count: 0,
            inbox_scroll_offset: 0,
            inbox_list_offset: 0,
            drafts: Vec::new(),
            drafts_selected: 0,
            drafts_list_offset: 0,
            drafts_error: None,
            needs_drafts_load: false,
            sent_emails: Vec::new(),
            sent_selected: 0,
            sent_view_mode: ViewMode::List,
            sent_loading: false,
            sent_error: None,
            sent_scroll_offset: 0,
            sent_list_offset: 0,
            compose_step: ComposeStep::Editing,
            compose_draft: EmailDraft::default(),
            compose_draft_path: None,
            signature_content: None,
            needs_signature_load: false,
            status_message: None,
            error_message: None,
            status_ttl: 0,
            auto_refresh_counter: 0,
            needs_inbox_cache_load: should_load_inbox,
            needs_inbox_load: should_load_inbox,
            cancel_inbox_load: false,
            inbox_open_loading: false,
            needs_sent_load: false,
            needs_editor_open: false,
            needs_full_redraw: false,
            needs_email_send: false,
            is_sending_email: false,
            pre_compose_screen: None,
            spinner_state: 0,
            pending_list_navigation: None,
        }
    }

    pub fn menu_items() -> Vec<&'static str> {
        vec!["Inbox", "Sent", "Drafts", "Signatures"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VeroConfig;

    #[test]
    fn app_new_defaults_to_all_filter() {
        let config = VeroConfig {
            accounts: Vec::new(),
            download_folder: None,
            inbox_view: "unseen".to_string(), // Should be overridden to All
            auto_refresh: crate::config::AutoRefresh { seconds: 0 },
            viewer: None,
            editor: None,
        };
        let app = App::new(config);
        assert_eq!(app.inbox_filter, InboxFilter::All);
    }
}
