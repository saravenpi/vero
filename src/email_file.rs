mod attachments;
mod draft;
mod fields;
mod parsed;
mod stored;

pub use draft::{
    create_draft_template, parse_draft_file_lenient, parse_email_file, validate_attachments,
};
pub use parsed::ParsedEmail;
pub use stored::{parse_stored_email_file, write_inbox_cache_email_file, write_sent_email_file};
