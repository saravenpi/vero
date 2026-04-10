use anyhow::{Context, Result};
use chrono::Utc;
use std::path::Path;

use crate::email_file::{parse_stored_email_file, write_sent_email_file};
use crate::models::Email;

use super::paths::{ensure_dir, get_sent_dir, is_eml_file, sanitize_email, TIME_FORMAT};

pub fn save_sent_email(account_email: &str, mut email: Email) -> Result<()> {
    let sent_path = get_sent_dir(account_email)?;
    ensure_dir(&sent_path)?;

    let timestamp = Utc::now();
    let to = email.to.clone().unwrap_or_default();
    let filename = format!(
        "{}-{}.eml",
        timestamp.format(TIME_FORMAT),
        sanitize_email(&to)
    );

    let file_path = sent_path.join(filename);
    email.from = "Me".to_string();
    email.timestamp = timestamp;

    let data = write_sent_email_file(&email)?;
    std::fs::write(file_path, data).context("Failed to write email file")?;

    Ok(())
}

pub fn load_sent_emails(account_email: &str) -> Result<Vec<Email>> {
    let sent_path = get_sent_dir(account_email)?;
    load_emails_from_dir(&sent_path)
}

pub fn delete_sent_email(account_email: &str, email: &Email) -> Result<()> {
    let sent_path = get_sent_dir(account_email)?;
    let timestamp_prefix = email.timestamp.format(TIME_FORMAT).to_string();

    for entry in std::fs::read_dir(&sent_path).context("Failed to read sent directory")? {
        let entry = entry?;
        let path = entry.path();
        if is_eml_file(&path, &entry) {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&timestamp_prefix))
                .unwrap_or(false)
            {
                std::fs::remove_file(&path).context("Failed to delete sent email")?;
                return Ok(());
            }
        }
    }

    anyhow::bail!("Sent email file not found for timestamp {}", timestamp_prefix)
}

fn load_emails_from_dir(dir_path: &Path) -> Result<Vec<Email>> {
    ensure_dir(dir_path)?;

    let entries = std::fs::read_dir(dir_path).context("Failed to read directory")?;

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
                Err(e) => {
                    errors.push(format!("Failed to parse {}: {}", path.display(), e));
                }
            },
            Err(e) => {
                errors.push(format!("Failed to read {}: {}", path.display(), e));
            }
        }
    }

    if !errors.is_empty() && emails.is_empty() {
        anyhow::bail!("Failed to load any emails. Errors: {}", errors.join("; "));
    }

    emails.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(emails)
}
