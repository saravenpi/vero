use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub from: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<String>,
    pub subject: String,
    pub date: String,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(skip)]
    pub uid: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxFilter {
    Unseen,
    Seen,
    All,
}

impl InboxFilter {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            InboxFilter::Unseen => "unseen",
            InboxFilter::Seen => "seen",
            InboxFilter::All => "all",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "unseen" => InboxFilter::Unseen,
            "seen" => InboxFilter::Seen,
            _ => InboxFilter::All,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EmailDraft {
    pub to: String,
    pub cc: String,
    pub bcc: String,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<Attachment>,
}
