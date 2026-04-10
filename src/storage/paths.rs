use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const VERO_DIR: &str = ".vero";
const INBOX_DIR: &str = "inbox";
const SEEN_DIR: &str = "seen";
const SENT_DIR: &str = "sent";
const DRAFTS_DIR: &str = "drafts";

pub(super) const TIME_FORMAT: &str = "%Y-%m-%d-%H%M%S";

pub(super) fn get_drafts_dir(account_email: &str) -> Result<PathBuf> {
    Ok(get_account_dir(account_email)?.join(DRAFTS_DIR))
}

pub(super) fn get_inbox_dir(account_email: &str) -> Result<PathBuf> {
    Ok(get_account_dir(account_email)?.join(INBOX_DIR))
}

pub(super) fn get_sent_dir(account_email: &str) -> Result<PathBuf> {
    Ok(get_account_dir(account_email)?.join(SENT_DIR))
}

pub(super) fn get_seen_dir(account_email: &str) -> Result<PathBuf> {
    Ok(get_account_dir(account_email)?.join(SEEN_DIR))
}

pub(super) fn get_signature_path(account_email: &str) -> Result<PathBuf> {
    Ok(get_account_dir(account_email)?.join("signature.txt"))
}

pub(super) fn ensure_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).context("Failed to create directory")?;
    Ok(())
}

pub(super) fn sanitize_email(email: &str) -> String {
    email.replace(['@', '.', '+', ' '], "_")
}

pub(super) fn is_eml_file(path: &Path, entry: &std::fs::DirEntry) -> bool {
    entry.file_type().map(|t| t.is_file()).unwrap_or(false)
        && path.extension() == Some(OsStr::new("eml"))
}

fn get_vero_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Unable to find home directory")?;
    Ok(home.join(VERO_DIR))
}

fn get_account_dir(account_email: &str) -> Result<PathBuf> {
    Ok(get_vero_dir()?.join(account_email))
}
