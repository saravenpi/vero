use std::path::PathBuf;

use chrono::Utc;

use crate::{
    config::{Account, AutoRefresh, ImapConfig, SmtpConfig, VeroConfig},
    models::{Email, EmailDraft},
    tui::App,
};

pub(crate) fn test_app() -> App {
    App::new(VeroConfig {
        accounts: vec![Account {
            email: "test@example.com".to_string(),
            imap: ImapConfig {
                user: Some("test@example.com".to_string()),
                password: "secret".to_string(),
                host: "imap.example.com".to_string(),
                port: 993,
            },
            smtp: SmtpConfig {
                user: Some("test@example.com".to_string()),
                password: "secret".to_string(),
                host: "smtp.example.com".to_string(),
                port: 465,
            },
        }],
        download_folder: None,
        inbox_view: "all".to_string(),
        auto_refresh: AutoRefresh { seconds: 0 },
        viewer: None,
        editor: None,
    })
}

pub(crate) fn test_email(uid: u32) -> Email {
    Email {
        from: "sender@example.com".to_string(),
        to: Some("test@example.com".to_string()),
        cc: None,
        bcc: None,
        subject: "Subject".to_string(),
        date: "Thu, 09 Apr 2026 12:00:00 +0000".to_string(),
        body: String::new(),
        timestamp: Utc::now(),
        attachments: Vec::new(),
        uid,
        is_seen: false,
    }
}

pub(crate) fn test_draft_index(index: usize) -> (PathBuf, EmailDraft) {
    (
        PathBuf::from(format!("/tmp/draft-{}.md", index)),
        EmailDraft {
            subject: format!("Draft {}", index),
            to: format!("test{}@example.com", index),
            ..EmailDraft::default()
        },
    )
}

pub(crate) fn test_draft_named(name: &str) -> (PathBuf, EmailDraft) {
    (
        PathBuf::from(format!("/tmp/{}.md", name)),
        EmailDraft {
            subject: name.to_string(),
            ..EmailDraft::default()
        },
    )
}
