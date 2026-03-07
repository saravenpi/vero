use anyhow::{Context, Result};
use lettre::message::{header, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::config::SmtpConfig;
use crate::models::EmailDraft;

pub async fn send_email(cfg: &SmtpConfig, from: &str, draft: EmailDraft) -> Result<()> {
    let from_mailbox: Mailbox = from.parse().context("Invalid from address")?;
    let to_mailbox: Mailbox = draft.to.parse().context("Invalid to address")?;

    let mut email_builder = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(&draft.subject);

    if !draft.cc.is_empty() {
        for cc_addr in draft.cc.split(',') {
            let cc_addr = cc_addr.trim();
            if !cc_addr.is_empty() {
                let cc_mailbox: Mailbox = cc_addr.parse().context("Invalid CC address")?;
                email_builder = email_builder.cc(cc_mailbox);
            }
        }
    }

    if !draft.bcc.is_empty() {
        for bcc_addr in draft.bcc.split(',') {
            let bcc_addr = bcc_addr.trim();
            if !bcc_addr.is_empty() {
                let bcc_mailbox: Mailbox = bcc_addr.parse().context("Invalid BCC address")?;
                email_builder = email_builder.bcc(bcc_mailbox);
            }
        }
    }

    let email = if draft.attachments.is_empty() {
        email_builder
            .header(header::ContentType::TEXT_PLAIN)
            .body(draft.body.clone())
            .context("Failed to build email")?
    } else {
        let mut multipart = MultiPart::mixed()
            .singlepart(SinglePart::plain(draft.body.clone()));

        for attachment in &draft.attachments {
            if let Some(file_path) = &attachment.file_path {
                let file_data = std::fs::read(file_path)
                    .with_context(|| format!("Failed to read attachment: {}", file_path))?;

                let content_type: header::ContentType = attachment.content_type.parse()
                    .unwrap_or_else(|_| "application/octet-stream".parse().unwrap());

                multipart = multipart.singlepart(
                    SinglePart::builder()
                        .header(content_type)
                        .header(header::ContentDisposition::attachment(&attachment.filename))
                        .body(file_data)
                );
            }
        }

        email_builder
            .multipart(multipart)
            .context("Failed to build email with attachments")?
    };

    let creds = Credentials::new(
        cfg.user.clone().unwrap_or_default(),
        cfg.password.clone(),
    );

    let mailer = SmtpTransport::relay(&cfg.host)
        .context("Failed to create SMTP transport")?
        .credentials(creds)
        .port(cfg.port)
        .build();

    tokio::task::spawn_blocking(move || {
        mailer.send(&email)
            .context("Failed to send email")
    })
    .await??;

    Ok(())
}
