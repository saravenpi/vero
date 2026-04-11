use anyhow::Result;
use mailparse::MailHeaderMap;

use crate::models::Attachment;

pub(super) fn extract_attachments_with_bytes(
    mail: &mailparse::ParsedMail,
) -> Result<Vec<(Attachment, Vec<u8>)>> {
    let mut result = Vec::new();
    collect_attachment_bytes(mail, &mut result)?;
    Ok(result)
}

pub(super) fn attachment_identity(mail: &mailparse::ParsedMail) -> Option<(String, String)> {
    let content_disposition = mail.headers.get_first_value("Content-Disposition");
    if !is_attachment(content_disposition.as_deref()) {
        return None;
    }

    let filename = content_disposition
        .as_deref()
        .and_then(extract_filename)
        .unwrap_or_else(|| "unknown".to_string());
    let content_type = mail.ctype.mimetype.to_ascii_lowercase();
    Some((filename, content_type))
}

fn collect_attachment_bytes(
    mail: &mailparse::ParsedMail,
    result: &mut Vec<(Attachment, Vec<u8>)>,
) -> Result<()> {
    if let Some((filename, content_type)) = attachment_identity(mail) {
        let bytes = mail.get_body_raw()?;
        let size = bytes.len() as i64;
        result.push((
            Attachment {
                filename,
                content_type,
                size,
                file_path: None,
            },
            bytes,
        ));
        return Ok(());
    }

    for subpart in &mail.subparts {
        collect_attachment_bytes(subpart, result)?;
    }

    Ok(())
}

fn is_attachment(content_disposition: Option<&str>) -> bool {
    content_disposition
        .map(|value| value.to_ascii_lowercase().contains("attachment"))
        .unwrap_or(false)
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
