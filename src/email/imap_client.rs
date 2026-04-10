mod body;
mod delete;
mod envelope;
mod fetch;
mod session;

pub use delete::delete_email;
pub use fetch::{fetch_attachment_bytes, fetch_email_body, fetch_emails, fetch_unseen_count};

const COMMAND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);
