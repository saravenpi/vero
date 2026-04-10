use std::path::PathBuf;

use crate::models::InboxFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone)]
pub struct CliInvocation {
    pub output: OutputFormat,
    pub account: Option<String>,
    pub command: CliCommand,
}

#[derive(Debug, Clone)]
pub enum CliCommand {
    Tui,
    Help,
    Version,
    AccountsList,
    Inbox(InboxCommand),
    Sent(SentCommand),
    Send(SendCommand),
    Draft(DraftCommand),
}

#[derive(Debug, Clone)]
pub enum InboxCommand {
    List {
        filter: Option<InboxFilter>,
        limit: Option<usize>,
    },
    Show {
        uid: u32,
    },
    Delete {
        uid: u32,
    },
    Download {
        uid: u32,
        index: Option<usize>,
    },
    UnreadCount,
}

#[derive(Debug, Clone)]
pub enum SentCommand {
    List { limit: Option<usize> },
    Show { index: usize },
}

#[derive(Debug, Clone)]
pub enum SendCommand {
    DraftFile {
        path: PathBuf,
    },
    Fields {
        to: String,
        cc: Option<String>,
        bcc: Option<String>,
        subject: String,
        body: SendBody,
        attachments: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum SendBody {
    Inline(String),
    File(PathBuf),
    Empty,
}

#[derive(Debug, Clone)]
pub enum DraftCommand {
    Template { output_path: Option<PathBuf> },
}
