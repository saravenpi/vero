pub(crate) mod date;
mod imap_client;
mod smtp_client;

pub use imap_client::delete_email;
pub use imap_client::fetch_email_body;
pub use imap_client::fetch_emails;
pub use imap_client::fetch_unseen_count;
pub use smtp_client::send_email;
