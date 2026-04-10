use anyhow::Result;
use std::path::PathBuf;

use super::paths::{ensure_dir, get_signature_path};

pub fn get_or_create_signature_path(account_email: &str) -> Result<PathBuf> {
    let path = get_signature_path(account_email)?;
    ensure_dir(path.parent().unwrap())?;
    if !path.exists() {
        std::fs::write(&path, "")?;
    }
    Ok(path)
}

pub fn load_signature(account_email: &str) -> Option<String> {
    let path = get_signature_path(account_email).ok()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}
