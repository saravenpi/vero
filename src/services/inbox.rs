use anyhow::{anyhow, Result};

use crate::config::Account;
use crate::models::{Email, InboxFilter};

pub struct InboxSnapshot {
    pub emails: Vec<Email>,
    pub unseen_count: usize,
}

pub async fn load_inbox(account: &Account, filter: InboxFilter) -> Result<InboxSnapshot> {
    let (emails, unseen_count) = tokio::try_join!(
        crate::email::fetch_emails(&account.imap, filter),
        crate::email::fetch_unseen_count(&account.imap)
    )?;

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
    let (body, attachments) = crate::email::fetch_email_body(&account.imap, email.uid).await?;

    email.body = body;
    email.attachments = attachments;

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
