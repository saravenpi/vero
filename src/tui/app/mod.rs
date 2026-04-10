mod inbox;
mod lifecycle;
mod list_navigation;
mod navigation;
mod selection;
mod state;

use crate::config::{Account, VeroConfig};
use crate::models::{Email, EmailDraft, InboxFilter, ViewMode};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    AccountSelection,
    Inbox,
    Drafts,
    Sent,
    Compose,
    Signatures,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedElement {
    MenuBar,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeStep {
    Editing,
    Preview,
    NoEditor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingListNavigation {
    AwaitingSecondG,
}

pub struct App {
    pub config: VeroConfig,
    pub current_account: Option<Account>,
    pub screen: Screen,
    pub focused: FocusedElement,

    pub account_selected: usize,

    pub menu_selected: usize,

    pub inbox_filter: InboxFilter,
    pub inbox_cached_emails: Vec<Email>,
    pub inbox_cache_loaded: bool,
    pub inbox_emails: Vec<Email>,
    pub inbox_selected: usize,
    pub inbox_view_mode: ViewMode,
    pub inbox_loading: bool,
    pub inbox_error: Option<String>,
    pub inbox_unseen_count: usize,
    pub inbox_scroll_offset: usize,
    pub inbox_list_offset: usize,

    pub drafts: Vec<(PathBuf, EmailDraft)>,
    pub drafts_selected: usize,
    pub drafts_list_offset: usize,
    pub drafts_error: Option<String>,
    pub needs_drafts_load: bool,

    pub sent_emails: Vec<Email>,
    pub sent_selected: usize,
    pub sent_view_mode: ViewMode,
    pub sent_loading: bool,
    pub sent_error: Option<String>,
    pub sent_scroll_offset: usize,
    pub sent_list_offset: usize,

    pub compose_step: ComposeStep,
    pub compose_draft: EmailDraft,
    pub compose_draft_path: Option<PathBuf>,

    pub signature_content: Option<String>,
    pub needs_signature_load: bool,

    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub status_ttl: u8,

    pub auto_refresh_counter: u64,

    pub needs_inbox_cache_load: bool,
    pub needs_inbox_load: bool,
    pub cancel_inbox_load: bool,
    pub inbox_open_loading: bool,
    pub needs_sent_load: bool,
    pub needs_editor_open: bool,
    pub needs_full_redraw: bool,
    pub needs_email_send: bool,
    pub is_sending_email: bool,
    pub pre_compose_screen: Option<Screen>,

    pub inbox_show_attachments: bool,
    pub inbox_attachment_selected: usize,
    pub needs_attachment_download: bool,
    pub is_downloading_attachment: bool,
    pub attachment_download_index: Option<usize>,

    pub spinner_state: usize,

    pending_list_navigation: Option<PendingListNavigation>,
}
