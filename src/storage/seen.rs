use anyhow::{Context, Result};
use chrono::Utc;

use crate::email_file::{parse_stored_email_file, write_sent_email_file};
use crate::models::Email;

use super::paths::{ensure_dir, get_seen_dir, is_eml_file, sanitize_email, TIME_FORMAT};

pub fn save_seen_email(account_email: &str, mut email: Email) -> Result<()> {
    let seen_path = get_seen_dir(account_email)?;
    ensure_dir(&seen_path)?;

    let timestamp = Utc::now();
    let filename = format!(
        "{}-{}.eml",
        timestamp.format(TIME_FORMAT),
        sanitize_email(&email.from)
    );

    let file_path = seen_path.join(filename);
    email.timestamp = timestamp;

    let data = write_sent_email_file(&email)?;
    std::fs::write(file_path, data).context("Failed to write email file")?;

    Ok(())
}

pub fn delete_seen_email(account_email: &str, email: &Email) -> Result<()> {
    let seen_path = get_seen_dir(account_email)?;

    if !seen_path.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(&seen_path).context("Failed to read seen directory")?;

    for entry in entries.flatten() {
        let path = entry.path();

        if !is_eml_file(&path, &entry) {
            continue;
        }

        let Ok(data) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(stored_email) = parse_stored_email_file(&data) else {
            continue;
        };

        if stored_email.from == email.from && stored_email.subject == email.subject {
            std::fs::remove_file(path)?;
            return Ok(());
        }
    }

    Ok(())
}
