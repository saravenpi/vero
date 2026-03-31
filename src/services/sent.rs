use anyhow::{anyhow, Result};

use crate::config::Account;
use crate::models::Email;

pub fn load_sent_emails(account: &Account) -> Result<Vec<Email>> {
    crate::storage::load_sent_emails(&account.email)
}

pub fn read_sent_email(account: &Account, index: usize) -> Result<Email> {
    let emails = load_sent_emails(account)?;
    emails
        .get(index.saturating_sub(1))
        .cloned()
        .ok_or_else(|| anyhow!("No sent email found at index {}", index))
}
