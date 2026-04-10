use anyhow::Result;
use html2text::from_read;
use mailparse::MailHeaderMap;
use std::char;

use crate::models::Attachment;

const HTML_RENDER_WIDTH: usize = 120;

#[derive(Default)]
struct ExtractedBodies {
    plain_parts: Vec<String>,
    html_parts: Vec<String>,
}

impl ExtractedBodies {
    fn from_plain(part: String) -> Self {
        let mut bodies = Self::default();
        if !part.is_empty() {
            bodies.plain_parts.push(part);
        }
        bodies
    }

    fn from_html(part: String) -> Self {
        let mut bodies = Self::default();
        if !part.is_empty() {
            bodies.html_parts.push(part);
        }
        bodies
    }

    fn append(&mut self, other: Self) {
        for part in other.plain_parts {
            push_unique_part(&mut self.plain_parts, part);
        }

        for part in other.html_parts {
            push_unique_part(&mut self.html_parts, part);
        }
    }

    fn has_plain(&self) -> bool {
        !self.plain_parts.is_empty()
    }

    fn has_html(&self) -> bool {
        !self.html_parts.is_empty()
    }
    fn into_terminal_body(self) -> Option<String> {
        join_body_parts(&self.plain_parts).or_else(|| join_body_parts(&self.html_parts))
    }
}

pub(super) fn extract_attachments_with_bytes(
    mail: &mailparse::ParsedMail,
) -> Result<Vec<(Attachment, Vec<u8>)>> {
    let mut result = Vec::new();
    collect_attachment_bytes(mail, &mut result)?;
    Ok(result)
}

fn collect_attachment_bytes(
    mail: &mailparse::ParsedMail,
    result: &mut Vec<(Attachment, Vec<u8>)>,
) -> Result<()> {
    let ctype = mail.ctype.mimetype.to_ascii_lowercase();
    let content_disposition = mail.headers.get_first_value("Content-Disposition");

    if is_attachment(content_disposition.as_deref()) {
        let filename = content_disposition
            .as_deref()
            .and_then(extract_filename)
            .unwrap_or_else(|| "unknown".to_string());
        let bytes = mail.get_body_raw()?;
        let size = bytes.len() as i64;
        result.push((
            Attachment {
                filename,
                content_type: ctype,
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

pub(super) fn extract_body_and_attachments(
    mail: &mailparse::ParsedMail,
) -> Result<(String, Vec<Attachment>)> {
    let mut attachments = Vec::new();
    let body = extract_parts(mail, &mut attachments)?
        .into_terminal_body()
        .unwrap_or_else(|| "No text content".to_string());

    Ok((body, attachments))
}

fn extract_parts(
    mail: &mailparse::ParsedMail,
    attachments: &mut Vec<Attachment>,
) -> Result<ExtractedBodies> {
    let ctype = mail.ctype.mimetype.to_ascii_lowercase();
    let content_disposition = mail.headers.get_first_value("Content-Disposition");

    if is_attachment(content_disposition.as_deref()) {
        let filename = content_disposition
            .as_deref()
            .and_then(extract_filename)
            .unwrap_or_else(|| "unknown".to_string());
        let size = mail.get_body_raw()?.len() as i64;

        attachments.push(Attachment {
            filename,
            content_type: ctype,
            size,
            file_path: None,
        });

        return Ok(ExtractedBodies::default());
    }

    if ctype == "multipart/alternative" {
        return extract_alternative_parts(mail, attachments);
    }

    if ctype.starts_with("multipart/") {
        return extract_multipart_parts(mail, attachments);
    }

    match ctype.as_str() {
        "text/plain" => Ok(ExtractedBodies::from_plain(
            extract_text_part(mail)
                .map(|text| normalize_plain_text(text.as_str()))
                .unwrap_or_default(),
        )),
        "text/html" => Ok(ExtractedBodies::from_html(
            extract_text_part(mail)
                .map(|html| render_html_to_terminal_text(html.as_str()))
                .unwrap_or_default(),
        )),
        _ => Ok(ExtractedBodies::default()),
    }
}

fn extract_multipart_parts(
    mail: &mailparse::ParsedMail,
    attachments: &mut Vec<Attachment>,
) -> Result<ExtractedBodies> {
    let mut combined = ExtractedBodies::default();

    for subpart in &mail.subparts {
        combined.append(extract_parts(subpart, attachments)?);
    }

    Ok(combined)
}

fn extract_alternative_parts(
    mail: &mailparse::ParsedMail,
    attachments: &mut Vec<Attachment>,
) -> Result<ExtractedBodies> {
    let mut preferred_plain = None;
    let mut preferred_html = None;

    for subpart in &mail.subparts {
        let extracted = extract_parts(subpart, attachments)?;
        if extracted.has_plain() {
            preferred_plain = Some(extracted);
        } else if extracted.has_html() {
            preferred_html = Some(extracted);
        }
    }

    Ok(preferred_plain.or(preferred_html).unwrap_or_default())
}

fn extract_text_part(mail: &mailparse::ParsedMail) -> Option<String> {
    let text = mail.get_body().ok()?;
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

fn join_body_parts(parts: &[String]) -> Option<String> {
    let merged = parts
        .iter()
        .filter(|part| !part.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n\n");

    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

fn push_unique_part(parts: &mut Vec<String>, part: String) {
    if parts.last().is_some_and(|existing| existing == &part) {
        return;
    }

    parts.push(part);
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

fn normalize_plain_text(text: &str) -> String {
    normalize_terminal_text(text, false)
}

fn render_html_to_terminal_text(html: &str) -> String {
    let rendered =
        from_read(html.as_bytes(), HTML_RENDER_WIDTH).unwrap_or_else(|_| strip_html_fallback(html));

    normalize_terminal_text(rendered.as_str(), false)
}

fn strip_html_fallback(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }

    normalize_terminal_text(decode_html_entities(text.as_str()).as_str(), true)
}

fn decode_html_entities(text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '&' {
            decoded.push(ch);
            continue;
        }

        let mut entity = String::new();
        let mut terminated = false;

        while let Some(&next) = chars.peek() {
            if next == ';' {
                entity.push(next);
                chars.next();
                terminated = true;
                break;
            }

            if next.is_whitespace() || next == '<' || entity.len() >= 16 {
                break;
            }

            entity.push(next);
            chars.next();
        }

        if terminated {
            if let Some(replacement) = decode_html_entity(entity.as_str()) {
                decoded.push_str(replacement.as_str());
                continue;
            }
        }

        decoded.push('&');
        decoded.push_str(entity.as_str());
    }

    decoded
}

fn decode_html_entity(entity: &str) -> Option<String> {
    let name = entity.strip_suffix(';')?;

    if let Some(hex) = name.strip_prefix("#x").or_else(|| name.strip_prefix("#X")) {
        return u32::from_str_radix(hex, 16)
            .ok()
            .and_then(char::from_u32)
            .map(|value| value.to_string());
    }

    if let Some(decimal) = name.strip_prefix('#') {
        return decimal
            .parse::<u32>()
            .ok()
            .and_then(char::from_u32)
            .map(|value| value.to_string());
    }

    match name {
        "nbsp" => Some(" ".to_string()),
        "lt" => Some("<".to_string()),
        "gt" => Some(">".to_string()),
        "amp" => Some("&".to_string()),
        "quot" => Some("\"".to_string()),
        "apos" | "#39" => Some("'".to_string()),
        "bull" | "middot" => Some("*".to_string()),
        "ndash" | "mdash" => Some("-".to_string()),
        "hellip" => Some("...".to_string()),
        _ => None,
    }
}

fn normalize_terminal_text(text: &str, collapse_inline_whitespace: bool) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = Vec::new();
    let mut previous_blank = false;

    for raw_line in normalized.split('\n') {
        let line = clean_terminal_line(raw_line, collapse_inline_whitespace);

        if line.trim().is_empty() {
            if !lines.is_empty() && !previous_blank {
                lines.push(String::new());
                previous_blank = true;
            }
            continue;
        }

        lines.push(line);
        previous_blank = false;
    }

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}

fn clean_terminal_line(line: &str, collapse_inline_whitespace: bool) -> String {
    let mut cleaned = String::with_capacity(line.len());
    let mut last_was_space = false;

    for ch in line.chars() {
        match ch {
            '\t' if collapse_inline_whitespace => {
                if !last_was_space {
                    cleaned.push(' ');
                    last_was_space = true;
                }
            }
            '\t' => {
                cleaned.push_str("    ");
                last_was_space = false;
            }
            '\u{00a0}' => {
                if !last_was_space {
                    cleaned.push(' ');
                    last_was_space = true;
                }
            }
            '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{feff}' => {}
            _ if ch.is_control() => {}
            _ if collapse_inline_whitespace && ch.is_whitespace() => {
                if !last_was_space {
                    cleaned.push(' ');
                    last_was_space = true;
                }
            }
            _ => {
                cleaned.push(ch);
                last_was_space = ch == ' ';
            }
        }
    }

    if collapse_inline_whitespace {
        cleaned.trim().to_string()
    } else {
        cleaned.trim_end().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_body_and_attachments, render_html_to_terminal_text};
    use mailparse::parse_mail;

    #[test]
    fn prefers_plain_text_over_html_parts() {
        let raw = concat!(
            "Content-Type: multipart/alternative; boundary=\"b\"\r\n",
            "\r\n",
            "--b\r\n",
            "Content-Type: text/plain; charset=UTF-8\r\n",
            "\r\n",
            "Hello from plain text.\r\n",
            "--b\r\n",
            "Content-Type: text/html; charset=UTF-8\r\n",
            "\r\n",
            "<html><body><p>Hello from <b>HTML</b>.</p></body></html>\r\n",
            "--b--\r\n"
        );

        let parsed = parse_mail(raw.as_bytes()).unwrap();
        let (body, attachments) = extract_body_and_attachments(&parsed).unwrap();

        assert_eq!(body, "Hello from plain text.");
        assert!(attachments.is_empty());
    }

    #[test]
    fn mixed_messages_keep_plain_cover_note_and_alternative_body() {
        let raw = concat!(
            "Content-Type: multipart/mixed; boundary=\"outer\"\r\n",
            "\r\n",
            "--outer\r\n",
            "Content-Type: text/plain; charset=UTF-8\r\n",
            "\r\n",
            "Cover note.\r\n",
            "--outer\r\n",
            "Content-Type: multipart/alternative; boundary=\"inner\"\r\n",
            "\r\n",
            "--inner\r\n",
            "Content-Type: text/plain; charset=UTF-8\r\n",
            "\r\n",
            "Body from plain text.\r\n",
            "--inner\r\n",
            "Content-Type: text/html; charset=UTF-8\r\n",
            "\r\n",
            "<html><body><p>Body from <b>HTML</b>.</p></body></html>\r\n",
            "--inner--\r\n",
            "--outer\r\n",
            "Content-Type: application/pdf\r\n",
            "Content-Disposition: attachment; filename=\"invoice.pdf\"\r\n",
            "\r\n",
            "fakepdf\r\n",
            "--outer--\r\n"
        );

        let parsed = parse_mail(raw.as_bytes()).unwrap();
        let (body, attachments) = extract_body_and_attachments(&parsed).unwrap();

        assert_eq!(body, "Cover note.\n\nBody from plain text.");
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].filename, "invoice.pdf");
    }

    #[test]
    fn html_fallback_renders_terminal_friendly_text() {
        let rendered = render_html_to_terminal_text(concat!(
            "<html><head><style>.hidden { display:none; }</style></head><body>",
            "<p>Hello&nbsp;world</p>",
            "<ul><li>One</li><li>Two</li></ul>",
            "<script>alert('x')</script>",
            "</body></html>"
        ));

        assert!(rendered.contains("Hello world"));
        assert!(rendered.contains("One"));
        assert!(rendered.contains("Two"));
        assert!(!rendered.contains("alert('x')"));
        assert!(!rendered.contains("display:none"));
    }

    #[test]
    fn html_fallback_decodes_entities() {
        let rendered =
            render_html_to_terminal_text("<div>&lt;hello&gt;</div><div>&#39;quoted&#39;</div>");

        assert!(rendered.contains("<hello>"));
        assert!(rendered.contains("'quoted'"));
    }
}
