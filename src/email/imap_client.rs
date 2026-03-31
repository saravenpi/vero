use anyhow::{Context, Result};
use async_imap::types::Fetch;
use async_native_tls::TlsConnector;
use async_std::net::TcpStream;
use chrono::{DateTime, Utc};
use futures::{pin_mut, StreamExt};
use mailparse::{parse_mail, MailHeaderMap};

use crate::config::ImapConfig;
use crate::models::{Attachment, Email, InboxFilter};

fn decode_mime_header(raw: &[u8]) -> String {
    let utf8_str = String::from_utf8_lossy(raw);
    let trimmed = utf8_str.trim();

    if trimmed.is_empty() {
        return String::new();
    }

    match encoded_words::decode(trimmed) {
        Ok(decoded) => decoded.decoded,
        Err(_) => trimmed.to_string(),
    }
}

pub async fn fetch_emails(cfg: &ImapConfig, filter: InboxFilter) -> Result<Vec<Email>> {
    let addr = format!("{}:{}", cfg.host, cfg.port);

    let tcp_stream = async_std::future::timeout(
        std::time::Duration::from_secs(10),
        TcpStream::connect(&addr),
    )
    .await
    .context("Connection timeout")?
    .context("Failed to connect to IMAP server")?;

    let tls = TlsConnector::new();
    let tls_stream = async_std::future::timeout(
        std::time::Duration::from_secs(10),
        tls.connect(&cfg.host, tcp_stream),
    )
    .await
    .context("TLS handshake timeout")?
    .context("TLS connection failed")?;

    let client = async_imap::Client::new(tls_stream);
    let mut session = client
        .login(cfg.user.as_ref().unwrap(), &cfg.password)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to login: {}", e.0))?;

    async_std::future::timeout(std::time::Duration::from_secs(10), session.select("INBOX"))
        .await
        .context("INBOX select timeout")??;

    let query = match filter {
        InboxFilter::Unseen => "UNSEEN",
        InboxFilter::Seen => "SEEN",
        InboxFilter::All => "ALL",
    };

    let uids = async_std::future::timeout(
        std::time::Duration::from_secs(10),
        session.uid_search(query),
    )
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

    let messages_stream = session.uid_fetch(&uid_str, "(UID ENVELOPE)").await?;

    let messages_vec: Vec<_> = async_std::future::timeout(
        std::time::Duration::from_secs(20),
        messages_stream.collect::<Vec<_>>(),
    )
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

fn parse_envelope(fetch: &Fetch) -> Option<Email> {
    let envelope = fetch.envelope()?;

    let from = envelope
        .from
        .as_ref()
        .and_then(|addrs| addrs.first())
        .map(format_address)
        .unwrap_or_default();

    let to = envelope.to.as_ref().map(|addrs| {
        addrs
            .iter()
            .map(format_address)
            .collect::<Vec<_>>()
            .join(", ")
    });

    let cc = envelope.cc.as_ref().map(|addrs| {
        addrs
            .iter()
            .map(format_address)
            .collect::<Vec<_>>()
            .join(", ")
    });

    let subject = envelope
        .subject
        .as_ref()
        .map(|s| decode_mime_header(s))
        .unwrap_or_default();

    let date = envelope
        .date
        .as_ref()
        .and_then(|d| String::from_utf8(d.to_vec()).ok())
        .unwrap_or_default();

    let timestamp = parse_date(&date).unwrap_or_else(Utc::now);

    Some(Email {
        from,
        to,
        cc,
        bcc: None,
        subject,
        date,
        body: String::new(),
        timestamp,
        attachments: Vec::new(),
        uid: fetch.uid.unwrap_or(0),
    })
}

fn format_address(addr: &async_imap::imap_proto::Address) -> String {
    let name = addr
        .name
        .as_ref()
        .map(|n| decode_mime_header(n))
        .filter(|n| !n.is_empty());

    let mailbox = addr
        .mailbox
        .as_ref()
        .and_then(|m| String::from_utf8(m.to_vec()).ok())
        .unwrap_or_default();

    let host = addr
        .host
        .as_ref()
        .and_then(|h| String::from_utf8(h.to_vec()).ok())
        .unwrap_or_default();

    let email = if !mailbox.is_empty() && !host.is_empty() {
        format!("{}@{}", mailbox, host)
    } else {
        String::new()
    };

    match name {
        Some(n) => format!("{} <{}>", n, email),
        _ => email,
    }
}

fn parse_date(date_str: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc2822(date_str)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            DateTime::parse_from_rfc3339(date_str)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
}

pub async fn fetch_email_body(cfg: &ImapConfig, uid: u32) -> Result<(String, Vec<Attachment>)> {
    let addr = format!("{}:{}", cfg.host, cfg.port);

    let tcp_stream = async_std::future::timeout(
        std::time::Duration::from_secs(10),
        TcpStream::connect(&addr),
    )
    .await
    .context("Connection timeout")?
    .context("Failed to connect to IMAP server")?;

    let tls = TlsConnector::new();
    let tls_stream = async_std::future::timeout(
        std::time::Duration::from_secs(10),
        tls.connect(&cfg.host, tcp_stream),
    )
    .await
    .context("TLS handshake timeout")?
    .context("TLS connection failed")?;

    let client = async_imap::Client::new(tls_stream);
    let mut session = client
        .login(cfg.user.as_ref().unwrap(), &cfg.password)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to login: {}", e.0))?;

    session.select("INBOX").await?;

    let mut messages = session.uid_fetch(format!("{}", uid), "BODY.PEEK[]").await?;

    let fetch = messages.next().await.context("No message found")??;

    let body_data = fetch.body().context("No body found")?;

    let parsed = parse_mail(body_data).context("Failed to parse email")?;

    let (body, attachments) = extract_body_and_attachments(&parsed)?;

    drop(messages);
    session.logout().await?;

    Ok((body, attachments))
}

fn extract_body_and_attachments(mail: &mailparse::ParsedMail) -> Result<(String, Vec<Attachment>)> {
    let mut body = String::new();
    let mut attachments = Vec::new();

    extract_parts(mail, &mut body, &mut attachments)?;

    if body.is_empty() {
        body = "No text content".to_string();
    }

    Ok((body, attachments))
}

fn extract_parts(
    mail: &mailparse::ParsedMail,
    body: &mut String,
    attachments: &mut Vec<Attachment>,
) -> Result<()> {
    let ctype = mail.ctype.mimetype.as_str();

    if ctype.starts_with("multipart/") {
        for subpart in &mail.subparts {
            extract_parts(subpart, body, attachments)?;
        }
    } else if ctype == "text/plain" || ctype == "text/html" {
        if let Ok(text) = mail.get_body() {
            if !text.trim().is_empty() {
                if !body.is_empty() {
                    body.push_str("\n\n");
                }
                if ctype == "text/html" {
                    body.push_str(&strip_html(&text));
                } else {
                    body.push_str(&text);
                }
            }
        }
    } else if let Some(content_disposition) = mail.headers.get_first_value("Content-Disposition") {
        if content_disposition.contains("attachment") {
            let filename =
                extract_filename(&content_disposition).unwrap_or_else(|| "unknown".to_string());

            let size = mail.get_body_raw()?.len() as i64;

            attachments.push(Attachment {
                filename,
                content_type: ctype.to_string(),
                size,
                file_path: None,
            });
        }
    }

    Ok(())
}

fn extract_filename(content_disposition: &str) -> Option<String> {
    for part in content_disposition.split(';') {
        let part = part.trim();
        if let Some(stripped) = part.strip_prefix("filename=") {
            return Some(stripped.trim_matches(|c| c == '"' || c == '\'').to_string());
        }
    }
    None
}

fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
}

pub async fn fetch_unseen_count(cfg: &ImapConfig) -> Result<usize> {
    let addr = format!("{}:{}", cfg.host, cfg.port);

    let tcp_stream = async_std::future::timeout(
        std::time::Duration::from_secs(10),
        TcpStream::connect(&addr),
    )
    .await
    .context("Connection timeout")?
    .context("Failed to connect to IMAP server")?;

    let tls = TlsConnector::new();
    let tls_stream = async_std::future::timeout(
        std::time::Duration::from_secs(10),
        tls.connect(&cfg.host, tcp_stream),
    )
    .await
    .context("TLS handshake timeout")?
    .context("TLS connection failed")?;

    let client = async_imap::Client::new(tls_stream);
    let mut session = client
        .login(cfg.user.as_ref().unwrap(), &cfg.password)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to login: {}", e.0))?;

    session.select("INBOX").await?;

    let uids = session.uid_search("UNSEEN").await?;
    let count = uids.len();

    session.logout().await?;

    Ok(count)
}

pub async fn delete_email(cfg: &ImapConfig, uid: u32) -> Result<()> {
    let addr = format!("{}:{}", cfg.host, cfg.port);
    let tcp_stream = TcpStream::connect(&addr).await?;
    let tls = TlsConnector::new();
    let tls_stream = tls.connect(&cfg.host, tcp_stream).await?;

    let client = async_imap::Client::new(tls_stream);
    let mut session = client
        .login(cfg.user.as_ref().unwrap(), &cfg.password)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to login: {}", e.0))?;

    session.select("INBOX").await?;

    let mut store_stream = session
        .uid_store(format!("{}", uid), "+FLAGS (\\Deleted)")
        .await?;

    while store_stream.next().await.is_some() {}
    drop(store_stream);

    let used_uid_expunge = match session.uid_expunge(format!("{}", uid)).await {
        Ok(expunge_stream) => {
            pin_mut!(expunge_stream);
            while expunge_stream.next().await.is_some() {}
            true
        }
        Err(_) => false,
    };

    if !used_uid_expunge {
        let expunge_stream = session.expunge().await?;
        pin_mut!(expunge_stream);
        while expunge_stream.next().await.is_some() {}
    }

    session.logout().await?;

    Ok(())
}
