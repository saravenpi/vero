use super::{App, ComposeStep, FocusedElement, ListSearch, Screen};
use crate::config::VeroConfig;
use crate::models::{EmailDraft, InboxFilter, ViewMode};

impl App {
    pub fn new(config: VeroConfig) -> Self {
        let inbox_filter = InboxFilter::from_str(&config.inbox_view);
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
            inbox_search: ListSearch::default(),
            drafts: Vec::new(),
            drafts_selected: 0,
            drafts_list_offset: 0,
            drafts_error: None,
            needs_drafts_load: false,
            drafts_search: ListSearch::default(),
            sent_emails: Vec::new(),
            sent_selected: 0,
            sent_view_mode: ViewMode::List,
            sent_loading: false,
            sent_error: None,
            sent_scroll_offset: 0,
            sent_list_offset: 0,
            sent_search: ListSearch::default(),
            compose_step: ComposeStep::Editing,
            compose_draft: EmailDraft::default(),
            compose_draft_path: None,
            compose_preview_scroll_offset: 0,
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
            inbox_show_attachments: false,
            inbox_attachment_selected: 0,
            needs_attachment_download: false,
            is_downloading_attachment: false,
            attachment_download_index: None,
            needs_inbox_open: false,
            inbox_open_pending_email: None,
            spinner_state: 0,
            pending_list_navigation: None,
            inbox_collapse_quotes: true,
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
    fn app_new_uses_configured_inbox_filter() {
        let config = VeroConfig {
            accounts: Vec::new(),
            download_folder: None,
            inbox_view: "unseen".to_string(),
            auto_refresh: crate::config::AutoRefresh { seconds: 0 },
            viewer: None,
            editor: None,
        };
        let app = App::new(config);
        assert_eq!(app.inbox_filter, InboxFilter::Unseen);
    }

    #[test]
    fn app_new_defaults_to_all_filter_when_not_configured() {
        let config = VeroConfig {
            accounts: Vec::new(),
            download_folder: None,
            inbox_view: String::new(),
            auto_refresh: crate::config::AutoRefresh { seconds: 0 },
            viewer: None,
            editor: None,
        };
        let app = App::new(config);
        assert_eq!(app.inbox_filter, InboxFilter::All);
    }

    #[test]
    fn screen_errors_do_not_expire_with_status_ttl() {
        let mut app = App::new(VeroConfig {
            accounts: Vec::new(),
            download_folder: None,
            inbox_view: String::new(),
            auto_refresh: crate::config::AutoRefresh { seconds: 0 },
            viewer: None,
            editor: None,
        });

        app.set_inbox_error("still broken");
        app.set_status("temporary");

        for _ in 0..30 {
            app.tick_spinner();
        }

        assert!(app.status_message.is_none());
        assert_eq!(app.inbox_error.as_deref(), Some("still broken"));
    }
}
