use anyhow::{anyhow, Context, Result};
use std::path::Path;

use crate::models::EmailDraft;

use super::attachments::parse_attachment_paths;
use super::fields::{parse_lenient_fields, parse_strict_fields};
use super::parsed::ParsedEmail;

pub fn parse_email_file(content: &str) -> Result<ParsedEmail> {
    let fields = parse_strict_fields(content)?;
    let headers = fields.headers;

    let to = headers
        .get("to")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("'to' field is required but empty or missing"))?;

    let subject = headers
        .get("subject")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("'subject' field is required but empty or missing"))?;

    let cc = headers.get("cc").cloned().filter(|s| !s.is_empty());
    let bcc = headers.get("bcc").cloned().filter(|s| !s.is_empty());

    let attachment_paths = headers
        .get("attachments")
        .map(|attachments| parse_attachment_paths(attachments))
        .unwrap_or_default();

    Ok(ParsedEmail {
        to,
        cc,
        bcc,
        subject,
        body: fields.body.trim().to_string(),
        attachment_paths,
    })
}

pub fn validate_attachments(attachment_paths: &[String]) -> Result<()> {
    for path in attachment_paths {
        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Err(anyhow!("Attachment file not found: {}", path));
        }

        if !path_obj.is_file() {
            return Err(anyhow!("Attachment path is not a file: {}", path));
        }

        std::fs::metadata(path)
            .with_context(|| format!("Cannot read attachment file: {}", path))?;
    }

    Ok(())
}

pub fn parse_draft_file_lenient(content: &str) -> EmailDraft {
    let fields = parse_lenient_fields(content);
    let headers = fields.headers;

    EmailDraft {
        to: headers.get("to").cloned().unwrap_or_default(),
        cc: headers.get("cc").cloned().unwrap_or_default(),
        bcc: headers.get("bcc").cloned().unwrap_or_default(),
        subject: headers.get("subject").cloned().unwrap_or_default(),
        body: fields.body.trim().to_string(),
        attachments: Vec::new(),
    }
}

pub fn create_draft_template(signature: Option<&str>) -> String {
    let body_content = match signature {
        Some(sig) => format!("\n\n-- \n{}", sig),
        None => String::new(),
    };
    format!(
        "to:\ncc:\nbcc:\nsubject:\nattachments:\nbody:{}",
        body_content
    )
}
