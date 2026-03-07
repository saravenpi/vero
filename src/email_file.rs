use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::path::Path;

use crate::models::{Attachment, Email, EmailDraft};

pub struct ParsedEmail {
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
    pub attachment_paths: Vec<String>,
}

impl ParsedEmail {
    pub fn to_draft(&self) -> EmailDraft {
        let attachments = self
            .attachment_paths
            .iter()
            .map(|path| {
                let filename = Path::new(path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                Attachment {
                    filename,
                    content_type: "application/octet-stream".to_string(),
                    size: 0,
                    file_path: Some(path.clone()),
                }
            })
            .collect();

        EmailDraft {
            to: self.to.clone(),
            cc: self.cc.clone().unwrap_or_default(),
            bcc: self.bcc.clone().unwrap_or_default(),
            subject: self.subject.clone(),
            body: self.body.clone(),
            attachments,
        }
    }
}

pub fn parse_email_file(content: &str) -> Result<ParsedEmail> {
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut body = String::new();
    let mut in_body = false;
    let mut body_started = false;

    for (line_num, line) in content.lines().enumerate() {
        if in_body {
            if body_started {
                body.push('\n');
            } else {
                body_started = true;
            }
            body.push_str(line);
        } else {
            if line.trim().is_empty() {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let field = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim();

                if field == "body" {
                    in_body = true;
                    if !value.is_empty() {
                        body.push_str(value);
                        body_started = true;
                    }
                } else {
                    headers.insert(field, value.to_string());
                }
            } else {
                return Err(anyhow!(
                    "Line {}: Missing ':' in header line: {}",
                    line_num + 1,
                    line
                ));
            }
        }
    }

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

    let attachment_paths = if let Some(attachments_str) = headers.get("attachments") {
        if attachments_str.trim().is_empty() {
            Vec::new()
        } else {
            attachments_str
                .split(',')
                .map(|s| {
                    let trimmed = s.trim();
                    if trimmed.starts_with('~') {
                        let home = dirs::home_dir().unwrap_or_default();
                        home.join(&trimmed[2..]).to_string_lossy().to_string()
                    } else {
                        trimmed.to_string()
                    }
                })
                .collect()
        }
    } else {
        Vec::new()
    };

    Ok(ParsedEmail {
        to,
        cc,
        bcc,
        subject,
        body: body.trim().to_string(),
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

pub fn create_draft_template() -> String {
    r#"to:
cc:
bcc:
subject:
attachments:
body:
"#
    .to_string()
}

pub fn write_sent_email_file(email: &Email) -> Result<String> {
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
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut body = String::new();
    let mut in_body = false;
    let mut body_started = false;

    for line in content.lines() {
        if in_body {
            if body_started {
                body.push('\n');
            } else {
                body_started = true;
            }
            body.push_str(line);
        } else {
            if line.trim().is_empty() {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let field = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim();

                if field == "body" {
                    in_body = true;
                    if !value.is_empty() {
                        body.push_str(value);
                        body_started = true;
                    }
                } else {
                    headers.insert(field, value.to_string());
                }
            }
        }
    }

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

    let attachments = if let Some(attachments_str) = headers.get("attachments") {
        attachments_str
            .split(',')
            .map(|s| Attachment {
                filename: s.trim().to_string(),
                content_type: "application/octet-stream".to_string(),
                size: 0,
                file_path: None,
            })
            .collect()
    } else {
        Vec::new()
    };

    let timestamp = chrono::Utc::now();

    Ok(Email {
        from,
        to,
        cc,
        bcc,
        subject,
        date,
        body: body.trim().to_string(),
        timestamp,
        attachments,
        uid: 0,
    })
}
