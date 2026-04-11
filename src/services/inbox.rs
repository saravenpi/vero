use anyhow::{anyhow, Result};

use crate::config::Account;
use crate::models::{Email, InboxFilter};

pub struct InboxSnapshot {
    pub emails: Vec<Email>,
    pub unseen_count: usize,
}

impl InboxSnapshot {
    pub fn filtered_emails(&self, filter: InboxFilter) -> Vec<Email> {
        self.emails
            .iter()
            .filter(|email| filter.matches(email))
            .cloned()
            .collect()
    }
}

pub async fn load_inbox(account: &Account) -> Result<InboxSnapshot> {
    let emails = crate::email::fetch_emails(&account.imap, InboxFilter::All).await?;
    let unseen_count = emails.iter().filter(|email| !email.is_seen).count();

    let _ = crate::storage::save_cached_inbox_emails(&account.email, &emails);

    Ok(InboxSnapshot {
        emails,
        unseen_count,
    })
}

pub fn load_cached_inbox(account: &Account) -> Result<InboxSnapshot> {
    let emails = crate::storage::load_cached_inbox_emails(&account.email)?;
    let unseen_count = emails.iter().filter(|email| !email.is_seen).count();

    Ok(InboxSnapshot {
        emails,
        unseen_count,
    })
}

pub async fn read_inbox_email(account: &Account, uid: u32) -> Result<Email> {
    let email = lookup_inbox_email(account, uid).await?;
    read_loaded_inbox_email(account, email).await
}

pub async fn read_loaded_inbox_email(account: &Account, mut email: Email) -> Result<Email> {
    let (body, attachments, references) =
        crate::email::fetch_email_body(&account.imap, email.uid).await?;

    email.body = body;
    email.attachments = attachments;
    if !references.is_empty() {
        email.references = references;
    }
    email.is_seen = true;

    crate::storage::save_seen_email(&account.email, email.clone())?;

    Ok(email)
}

pub async fn delete_inbox_email(account: &Account, uid: u32) -> Result<()> {
    match lookup_inbox_email(account, uid).await {
        Ok(email) => delete_loaded_inbox_email(account, &email).await,
        Err(_) => crate::email::delete_email(&account.imap, uid).await,
    }
}

pub async fn delete_loaded_inbox_email(account: &Account, email: &Email) -> Result<()> {
    crate::email::delete_email(&account.imap, email.uid).await?;
    crate::storage::delete_seen_email(&account.email, email)?;
    Ok(())
}

pub async fn unread_count(account: &Account) -> Result<usize> {
    crate::email::fetch_unseen_count(&account.imap).await
}

async fn lookup_inbox_email(account: &Account, uid: u32) -> Result<Email> {
    crate::email::fetch_emails(&account.imap, InboxFilter::All)
        .await?
        .into_iter()
        .find(|email| email.uid == uid)
        .ok_or_else(|| anyhow!("No inbox email found with uid {}", uid))
}
