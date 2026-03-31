use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::cli::types::OutputFormat;
use crate::models::{Attachment, Email, InboxFilter};
use crate::services::AccountSummary;

pub fn print_accounts(output: OutputFormat, accounts: &[AccountSummary]) -> Result<()> {
    match output {
        OutputFormat::Text => {
            for account in accounts {
                println!("{} {}", account.index, account.email);
            }
            Ok(())
        }
        OutputFormat::Json => print_json(&AccountsResponse { accounts }),
    }
}

pub fn print_inbox_list(
    output: OutputFormat,
    filter: InboxFilter,
    unseen_count: usize,
    emails: &[Email],
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("filter: {}", filter.as_str());
            println!("unread: {}", unseen_count);
            for email in emails {
                println!(
                    "{} | {} | {} | {}",
                    email.uid, email.date, email.from, email.subject
                );
            }
            Ok(())
        }
        OutputFormat::Json => print_json(&InboxListResponse {
            filter: filter.as_str(),
            unread_count: unseen_count,
            emails,
        }),
    }
}

pub fn print_email(output: OutputFormat, email: &Email, index: Option<usize>) -> Result<()> {
    match output {
        OutputFormat::Text => {
            if let Some(index) = index {
                println!("index: {}", index);
            }
            if email.uid != 0 {
                println!("uid: {}", email.uid);
            }
            println!("from: {}", email.from);
            if let Some(to) = &email.to {
                println!("to: {}", to);
            }
            if let Some(cc) = &email.cc {
                if !cc.is_empty() {
                    println!("cc: {}", cc);
                }
            }
            if let Some(bcc) = &email.bcc {
                if !bcc.is_empty() {
                    println!("bcc: {}", bcc);
                }
            }
            println!("subject: {}", email.subject);
            println!("date: {}", email.date);
            if email.attachments.is_empty() {
                println!("attachments: 0");
            } else {
                println!("attachments:");
                for attachment in &email.attachments {
                    println!("  {}", format_attachment(attachment));
                }
            }
            println!();
            println!("{}", email.body);
            Ok(())
        }
        OutputFormat::Json => print_json(&EmailResponse { index, email }),
    }
}

pub fn print_unread_count(output: OutputFormat, unread_count: usize) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("{}", unread_count);
            Ok(())
        }
        OutputFormat::Json => print_json(&UnreadCountResponse { unread_count }),
    }
}

pub fn print_deleted(output: OutputFormat, uid: u32) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("deleted {}", uid);
            Ok(())
        }
        OutputFormat::Json => print_json(&DeleteResponse { deleted_uid: uid }),
    }
}

pub fn print_sent(output: OutputFormat, emails: &[Email]) -> Result<()> {
    match output {
        OutputFormat::Text => {
            for (index, email) in emails.iter().enumerate() {
                println!(
                    "{} | {} | {} | {}",
                    index + 1,
                    email.date,
                    email.to.as_deref().unwrap_or(""),
                    email.subject
                );
            }
            Ok(())
        }
        OutputFormat::Json => {
            let emails = emails
                .iter()
                .enumerate()
                .map(|(index, email)| IndexedEmail {
                    index: index + 1,
                    email,
                })
                .collect::<Vec<_>>();
            print_json(&SentListResponse { emails })
        }
    }
}

pub fn print_sent_email(output: OutputFormat, index: usize, email: &Email) -> Result<()> {
    print_email(output, email, Some(index))
}

pub fn print_send_result(output: OutputFormat, email: &Email) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!(
                "sent {} -> {} | {}",
                email.from,
                email.to.as_deref().unwrap_or(""),
                email.subject
            );
            Ok(())
        }
        OutputFormat::Json => print_json(&SendResponse { email }),
    }
}

pub fn print_template(
    output: OutputFormat,
    template: &str,
    output_path: Option<&Path>,
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            if let Some(path) = output_path {
                println!("wrote {}", path.display());
            } else {
                print!("{}", template);
            }
            Ok(())
        }
        OutputFormat::Json => print_json(&TemplateResponse {
            output_path: output_path.map(|path| path.display().to_string()),
            template: output_path.is_none().then_some(template.to_string()),
        }),
    }
}

fn format_attachment(attachment: &Attachment) -> String {
    if attachment.size > 0 {
        format!(
            "{} ({}, {} bytes)",
            attachment.filename, attachment.content_type, attachment.size
        )
    } else {
        format!("{} ({})", attachment.filename, attachment.content_type)
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[derive(Serialize)]
struct AccountsResponse<'a> {
    accounts: &'a [AccountSummary],
}

#[derive(Serialize)]
struct InboxListResponse<'a> {
    filter: &'static str,
    unread_count: usize,
    emails: &'a [Email],
}

#[derive(Serialize)]
struct EmailResponse<'a> {
    index: Option<usize>,
    email: &'a Email,
}

#[derive(Serialize)]
struct UnreadCountResponse {
    unread_count: usize,
}

#[derive(Serialize)]
struct DeleteResponse {
    deleted_uid: u32,
}

#[derive(Serialize)]
struct IndexedEmail<'a> {
    index: usize,
    email: &'a Email,
}

#[derive(Serialize)]
struct SentListResponse<'a> {
    emails: Vec<IndexedEmail<'a>>,
}

#[derive(Serialize)]
struct SendResponse<'a> {
    email: &'a Email,
}

#[derive(Serialize)]
struct TemplateResponse {
    output_path: Option<String>,
    template: Option<String>,
}
