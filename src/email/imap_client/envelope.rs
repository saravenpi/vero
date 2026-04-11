use async_imap::types::{Fetch, Flag};
use chrono::Utc;

use crate::email::date::parse_email_timestamp;
use crate::models::Email;

pub(super) fn parse_envelope(fetch: &Fetch) -> Option<Email> {
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

    let timestamp = parse_email_timestamp(&date).unwrap_or_else(Utc::now);

    let message_id = envelope
        .message_id
        .as_ref()
        .and_then(|b| String::from_utf8(b.to_vec()).ok())
        .map(|s| s.trim().trim_matches('<').trim_matches('>').to_string())
        .filter(|s| !s.is_empty());

    let in_reply_to = envelope
        .in_reply_to
        .as_ref()
        .and_then(|b| String::from_utf8(b.to_vec()).ok())
        .map(|s| s.trim().trim_matches('<').trim_matches('>').to_string())
        .filter(|s| !s.is_empty());

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
        is_seen: fetch.flags().any(|flag| flag == Flag::Seen),
        message_id,
        in_reply_to,
        references: Vec::new(),
    })
}

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
