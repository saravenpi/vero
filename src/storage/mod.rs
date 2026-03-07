use anyhow::{Context, Result};
use chrono::Utc;
use std::path::PathBuf;

use crate::email_file::{parse_stored_email_file, write_sent_email_file};
use crate::models::Email;

const VERO_DIR: &str = ".vero";
const SEEN_DIR: &str = "seen";
const SENT_DIR: &str = "sent";
const DRAFTS_DIR: &str = "drafts";
const TIME_FORMAT: &str = "%Y-%m-%d-%H%M%S";

pub fn get_vero_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Unable to find home directory")?;
    Ok(home.join(VERO_DIR))
}

pub fn get_account_dir(account_email: &str) -> Result<PathBuf> {
    Ok(get_vero_dir()?.join(account_email))
}

pub fn get_drafts_dir(account_email: &str) -> Result<PathBuf> {
    Ok(get_account_dir(account_email)?.join(DRAFTS_DIR))
}

pub fn get_sent_dir(account_email: &str) -> Result<PathBuf> {
    Ok(get_account_dir(account_email)?.join(SENT_DIR))
}

pub fn get_seen_dir(account_email: &str) -> Result<PathBuf> {
    Ok(get_account_dir(account_email)?.join(SEEN_DIR))
}

fn ensure_dir(path: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(path).context("Failed to create directory")?;
    Ok(())
}

fn sanitize_email(email: &str) -> String {
    email.replace(['@', '.', '+', ' '], "_")
}

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

pub fn load_emails_from_dir(dir_path: &PathBuf) -> Result<Vec<Email>> {
    ensure_dir(dir_path)?;

    let entries = std::fs::read_dir(dir_path).context("Failed to read directory")?;

    let mut emails = Vec::new();
    let mut errors = Vec::new();

    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "eml" {
                        match std::fs::read_to_string(entry.path()) {
                            Ok(data) => match parse_stored_email_file(&data) {
                                Ok(email) => emails.push(email),
                                Err(e) => {
                                    errors.push(format!(
                                        "Failed to parse {}: {}",
                                        entry.path().display(),
                                        e
                                    ));
                                }
                            },
                            Err(e) => {
                                errors.push(format!(
                                    "Failed to read {}: {}",
                                    entry.path().display(),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if !errors.is_empty() && emails.is_empty() {
        anyhow::bail!("Failed to load any emails. Errors: {}", errors.join("; "));
    }

    emails.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(emails)
}

pub fn load_sent_emails(account_email: &str) -> Result<Vec<Email>> {
    let sent_path = get_sent_dir(account_email)?;
    load_emails_from_dir(&sent_path)
}

pub fn delete_seen_email(account_email: &str, email: &Email) -> Result<()> {
    let seen_path = get_seen_dir(account_email)?;

    let entries = std::fs::read_dir(&seen_path).context("Failed to read seen directory")?;

    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "eml" {
                        if let Ok(data) = std::fs::read_to_string(entry.path()) {
                            if let Ok(stored_email) = parse_stored_email_file(&data) {
                                if stored_email.from == email.from
                                    && stored_email.subject == email.subject
                                {
                                    std::fs::remove_file(entry.path())?;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn create_draft_file(account_email: &str) -> Result<PathBuf> {
    let drafts_path = get_drafts_dir(account_email)?;
    ensure_dir(&drafts_path)?;

    let timestamp = Utc::now();
    let filename = format!("{}.eml", timestamp.format(TIME_FORMAT));
    let file_path = drafts_path.join(filename);

    let template = crate::email_file::create_draft_template();
    std::fs::write(&file_path, template).context("Failed to create draft file")?;

    Ok(file_path)
}

pub fn delete_draft_file(draft_path: &PathBuf) -> Result<()> {
    if draft_path.exists() {
        std::fs::remove_file(draft_path).context("Failed to delete draft file")?;
    }
    Ok(())
}
