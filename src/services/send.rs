use anyhow::{Context, Result};
use chrono::Utc;
use std::path::Path;

use crate::config::Account;
use crate::models::{Attachment, Email, EmailDraft};

pub fn build_draft(
    to: String,
    cc: Option<String>,
    bcc: Option<String>,
    subject: String,
    body: String,
    attachments: Vec<String>,
) -> Result<EmailDraft> {
    crate::email_file::validate_attachments(&attachments)?;

    let attachments = attachments
        .into_iter()
        .map(build_attachment)
        .collect::<Result<Vec<_>>>()?;

    Ok(EmailDraft {
        to,
        cc: cc.unwrap_or_default(),
        bcc: bcc.unwrap_or_default(),
        subject,
        body,
        attachments,
    })
}

pub async fn send_draft(account: &Account, draft: EmailDraft) -> Result<Email> {
    crate::email::send_email(&account.smtp, &account.email, draft.clone()).await?;

    let email = Email {
        from: account.email.clone(),
        to: Some(draft.to.clone()),
        cc: (!draft.cc.is_empty()).then_some(draft.cc.clone()),
        bcc: (!draft.bcc.is_empty()).then_some(draft.bcc.clone()),
        subject: draft.subject.clone(),
        date: Utc::now().to_rfc2822(),
        body: draft.body.clone(),
        timestamp: Utc::now(),
        attachments: draft.attachments.clone(),
        uid: 0,
        is_seen: true,
    };

    crate::storage::save_sent_email(&account.email, email.clone())?;

    Ok(email)
}

fn build_attachment(path: String) -> Result<Attachment> {
    let file_name = Path::new(&path)
        .file_name()
        .context("Attachment path is missing a filename")?
        .to_string_lossy()
        .to_string();

    let size = std::fs::metadata(&path)
        .with_context(|| format!("Failed to stat attachment {}", path))?
        .len() as i64;

    Ok(Attachment {
        filename: file_name,
        content_type: "application/octet-stream".to_string(),
        size,
        file_path: Some(path),
    })
}
