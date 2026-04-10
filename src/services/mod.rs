mod accounts;
mod attachments;
mod drafts;
mod inbox;
mod send;
mod sent;

pub use accounts::{list_accounts, resolve_account, AccountSummary};
pub use attachments::download_inbox_attachments;
pub use drafts::{create_template, parse_draft_input, read_text_input, write_template};
pub use inbox::{
    delete_inbox_email, delete_loaded_inbox_email, load_cached_inbox, load_inbox, read_inbox_email,
    read_loaded_inbox_email, unread_count, InboxSnapshot,
};
pub use send::{build_draft, send_draft};
pub use sent::{load_sent_emails, read_sent_email};
