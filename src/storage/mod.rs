mod drafts;
mod inbox;
mod paths;
mod seen;
mod sent;
mod signatures;

pub use drafts::{create_draft_file, delete_draft_file, load_drafts};
pub use inbox::{load_cached_inbox_emails, save_cached_inbox_emails};
pub use seen::{delete_seen_email, save_seen_email};
pub use sent::{delete_sent_email, load_sent_emails, save_sent_email};
pub use signatures::{get_or_create_signature_path, load_signature};
