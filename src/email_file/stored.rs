use anyhow::Result;
use chrono::Utc;

use crate::email::date::parse_email_timestamp;
use crate::models::{Attachment, Email};

use super::fields::parse_lenient_fields;

pub fn write_sent_email_file(email: &Email) -> Result<String> {
    write_email_file(email, false)
}

pub fn write_inbox_cache_email_file(email: &Email) -> Result<String> {
    write_email_file(email, true)
}

fn write_email_file(email: &Email, include_inbox_fields: bool) -> Result<String> {
    let mut content = String::new();

    content.push_str(&format!("from: {}\n", email.from));

    if let Some(ref to) = email.to {
        content.push_str(&format!("to: {}\n", to));
    }

    if let Some(ref cc) = email.cc {
        if !cc.is_empty() {
            content.push_str(&format!("cc: {}\n", cc));
        }
    }

    if let Some(ref bcc) = email.bcc {
        if !bcc.is_empty() {
            content.push_str(&format!("bcc: {}\n", bcc));
        }
    }

    content.push_str(&format!("subject: {}\n", email.subject));
    content.push_str(&format!("date: {}\n", email.date));

    if include_inbox_fields {
        if email.uid != 0 {
            content.push_str(&format!("uid: {}\n", email.uid));
        }
        content.push_str(&format!("seen: {}\n", email.is_seen));
    }

    if !email.attachments.is_empty() {
        let attachment_names: Vec<String> = email
            .attachments
            .iter()
            .map(|a| a.filename.clone())
            .collect();
        content.push_str(&format!("attachments: {}\n", attachment_names.join(", ")));
    }

    content.push_str("body: ");
    content.push_str(&email.body);

    Ok(content)
}

pub fn parse_stored_email_file(content: &str) -> Result<Email> {
    let fields = parse_lenient_fields(content);
    let headers = fields.headers;

    let from = headers
        .get("from")
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    let to = headers.get("to").cloned();
    let cc = headers.get("cc").cloned();
    let bcc = headers.get("bcc").cloned();

    let subject = headers
        .get("subject")
        .cloned()
        .unwrap_or_else(|| "(No Subject)".to_string());

    let date = headers
        .get("date")
        .cloned()
        .unwrap_or_else(|| chrono::Utc::now().to_rfc2822());

    let timestamp = parse_email_timestamp(&date).unwrap_or_else(Utc::now);
    let uid = headers
        .get("uid")
        .and_then(|uid| uid.parse::<u32>().ok())
        .unwrap_or_default();
    let is_seen = headers
        .get("seen")
        .map(|seen| matches!(seen.as_str(), "true" | "1" | "yes"))
        .unwrap_or(true);

    let attachments = headers
        .get("attachments")
        .map(|attachments| {
            attachments
                .split(',')
                .map(|s| Attachment {
                    filename: s.trim().to_string(),
                    content_type: "application/octet-stream".to_string(),
                    size: 0,
                    file_path: None,
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Email {
        from,
        to,
        cc,
        bcc,
        subject,
        date,
        body: fields.body.trim().to_string(),
        timestamp,
        attachments,
        uid,
        is_seen,
        message_id: None,
        in_reply_to: None,
        references: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_stored_email_file;

    #[test]
    fn parses_timestamp_from_date_header() {
        let email = parse_stored_email_file(
            "from: Me\nto: test@example.com\nsubject: Test\ndate: Tue, 31 Mar 2026 15:00:33 +0000\nbody: hello",
        )
        .unwrap();

        assert_eq!(email.timestamp.to_rfc3339(), "2026-03-31T15:00:33+00:00");
    }

    #[test]
    fn parses_inbox_cache_metadata() {
        let email = parse_stored_email_file(
            "from: Me\nto: test@example.com\nsubject: Test\ndate: Tue, 31 Mar 2026 15:00:33 +0000\nuid: 42\nseen: false\nbody: hello",
        )
        .unwrap();

        assert_eq!(email.uid, 42);
        assert!(!email.is_seen);
    }
}
