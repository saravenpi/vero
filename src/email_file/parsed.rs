use std::path::Path;

use crate::models::{Attachment, EmailDraft};

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
