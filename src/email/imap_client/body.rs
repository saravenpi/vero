mod attachments;
mod html;
mod text;

use anyhow::Result;

use crate::models::Attachment;

use attachments::attachment_identity;
use html::render_html_to_terminal_text;
use text::{extract_text_part, join_body_parts, normalize_plain_text, push_unique_part};

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
    attachments::extract_attachments_with_bytes(mail)
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

    if let Some((filename, content_type)) = attachment_identity(mail) {
        let size = mail.get_body_raw()?.len() as i64;
        attachments.push(Attachment {
            filename,
            content_type,
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
