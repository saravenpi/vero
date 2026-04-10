use anyhow::{Context, Result};

use crate::email_file::{parse_stored_email_file, write_inbox_cache_email_file};
use crate::models::Email;

use super::paths::{ensure_dir, get_inbox_dir, is_eml_file};

pub fn save_cached_inbox_emails(account_email: &str, emails: &[Email]) -> Result<()> {
    let inbox_path = get_inbox_dir(account_email)?;
    ensure_dir(&inbox_path)?;

    for entry in std::fs::read_dir(&inbox_path).context("Failed to read inbox cache directory")? {
        let entry = entry?;
        let path = entry.path();

        if is_eml_file(&path, &entry) {
            std::fs::remove_file(path).context("Failed to remove stale inbox cache file")?;
        }
    }

    for (index, email) in emails.iter().enumerate() {
        let filename = format!("{:05}-{}.eml", index, email.uid);
        let path = inbox_path.join(filename);
        let data = write_inbox_cache_email_file(email)?;
        std::fs::write(path, data).context("Failed to write inbox cache file")?;
    }

    Ok(())
}

pub fn load_cached_inbox_emails(account_email: &str) -> Result<Vec<Email>> {
    let inbox_path = get_inbox_dir(account_email)?;
    ensure_dir(&inbox_path)?;

    let entries = std::fs::read_dir(&inbox_path).context("Failed to read inbox cache directory")?;
    let mut emails = Vec::new();
    let mut errors = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if !is_eml_file(&path, &entry) {
            continue;
        }

        match std::fs::read_to_string(&path) {
            Ok(data) => match parse_stored_email_file(&data) {
                Ok(email) => emails.push(email),
                Err(error) => {
                    errors.push(format!("Failed to parse {}: {}", path.display(), error));
                }
            },
            Err(error) => {
                errors.push(format!("Failed to read {}: {}", path.display(), error));
            }
        }
    }

    if !errors.is_empty() && emails.is_empty() {
        anyhow::bail!(
            "Failed to load any cached inbox emails: {}",
            errors.join("; ")
        );
    }

    emails.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(emails)
}
