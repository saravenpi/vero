use anyhow::{Context, Result};
use futures::StreamExt;
use mailparse::parse_mail;

use crate::config::ImapConfig;
use crate::models::{Attachment, Email, InboxFilter};

use super::body::extract_body_and_attachments;
use super::envelope::parse_envelope;
use super::session::login;
use super::{COMMAND_TIMEOUT, FETCH_TIMEOUT};

pub async fn fetch_emails(cfg: &ImapConfig, filter: InboxFilter) -> Result<Vec<Email>> {
    let mut session = login(cfg).await?;

    async_std::future::timeout(COMMAND_TIMEOUT, session.select("INBOX"))
        .await
        .context("INBOX select timeout")??;

    let query = match filter {
        InboxFilter::Unseen => "UNSEEN",
        InboxFilter::Seen => "SEEN",
        InboxFilter::All => "ALL",
    };

    let uids = async_std::future::timeout(COMMAND_TIMEOUT, session.uid_search(query))
        .await
        .context("Search timeout")??;

    if uids.is_empty() {
        let _ = session.logout().await;
        return Ok(Vec::new());
    }

    let uid_str = uids
        .iter()
        .map(|u| u.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let messages_stream = session.uid_fetch(&uid_str, "(UID FLAGS ENVELOPE)").await?;

    let messages_vec: Vec<_> =
        async_std::future::timeout(FETCH_TIMEOUT, messages_stream.collect::<Vec<_>>())
            .await
            .context("Failed to collect messages")?;

    let mut emails = Vec::new();
    for fetch in messages_vec.into_iter().flatten() {
        if let Some(email) = parse_envelope(&fetch) {
            emails.push(email);
        }
    }

    let _ = session.logout().await;

    emails.reverse();
    Ok(emails)
}

pub async fn fetch_email_body(cfg: &ImapConfig, uid: u32) -> Result<(String, Vec<Attachment>)> {
    let mut session = login(cfg).await?;

    session.select("INBOX").await?;

    let mut messages = session.uid_fetch(uid.to_string(), "BODY[]").await?;

    let fetch = messages.next().await.context("No message found")??;

    let body_data = fetch.body().context("No body found")?;

    let parsed = parse_mail(body_data).context("Failed to parse email")?;

    let (body, attachments) = extract_body_and_attachments(&parsed)?;

    drop(messages);
    session.logout().await?;

    Ok((body, attachments))
}

pub async fn fetch_unseen_count(cfg: &ImapConfig) -> Result<usize> {
    let mut session = login(cfg).await?;

    session.select("INBOX").await?;

    let uids = session.uid_search("UNSEEN").await?;
    let count = uids.len();

    session.logout().await?;

    Ok(count)
}
