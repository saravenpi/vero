use anyhow::Result;
use mailparse::MailHeaderMap;

use crate::models::Attachment;

pub(super) fn extract_body_and_attachments(
    mail: &mailparse::ParsedMail,
) -> Result<(String, Vec<Attachment>)> {
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
