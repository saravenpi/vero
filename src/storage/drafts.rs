use anyhow::{Context, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};

use crate::email_file::{create_draft_template, parse_draft_file_lenient};
use crate::models::EmailDraft;

use super::paths::{ensure_dir, get_drafts_dir, is_eml_file, TIME_FORMAT};
use super::signatures::load_signature;

pub fn create_draft_file(account_email: &str) -> Result<PathBuf> {
    let drafts_path = get_drafts_dir(account_email)?;
    ensure_dir(&drafts_path)?;

    let timestamp = Utc::now();
    let filename = format!("{}.eml", timestamp.format(TIME_FORMAT));
    let file_path = drafts_path.join(filename);

    let signature = load_signature(account_email);
    std::fs::write(&file_path, create_draft_template(signature.as_deref()))
        .context("Failed to create draft file")?;

    Ok(file_path)
}

pub fn load_drafts(account_email: &str) -> Result<Vec<(PathBuf, EmailDraft)>> {
    let drafts_path = get_drafts_dir(account_email)?;
    ensure_dir(&drafts_path)?;

    let entries = std::fs::read_dir(&drafts_path).context("Failed to read drafts directory")?;

    let mut drafts = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if !is_eml_file(&path, &entry) {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&path) {
            let draft = parse_draft_file_lenient(&content);
            drafts.push((path, draft));
        }
    }

    drafts.sort_by(|a, b| b.0.cmp(&a.0));

    Ok(drafts)
}

pub fn delete_draft_file(draft_path: &Path) -> Result<()> {
    if draft_path.exists() {
        std::fs::remove_file(draft_path).context("Failed to delete draft file")?;
    }
    Ok(())
}
