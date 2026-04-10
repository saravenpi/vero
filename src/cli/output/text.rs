use std::path::Path;

use crate::models::{Attachment, Email, InboxFilter};
use crate::services::AccountSummary;

pub(super) fn print_accounts(accounts: &[AccountSummary]) {
    for account in accounts {
        println!("{} {}", account.index, account.email);
    }
}

pub(super) fn print_inbox_list(filter: InboxFilter, unseen_count: usize, emails: &[Email]) {
    println!("filter: {}", filter.as_str());
    println!("unread: {}", unseen_count);
    for email in emails {
        println!(
            "{} | {} | {} | {}",
            email.uid, email.date, email.from, email.subject
        );
    }
}

pub(super) fn print_email(email: &Email, index: Option<usize>) {
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
    print_attachments(&email.attachments);
    println!();
    println!("{}", email.body);
}

pub(super) fn print_unread_count(unread_count: usize) {
    println!("{}", unread_count);
}

pub(super) fn print_deleted(uid: u32) {
    println!("deleted {}", uid);
}

pub(super) fn print_sent(emails: &[Email]) {
    for (index, email) in emails.iter().enumerate() {
        println!(
            "{} | {} | {} | {}",
            index + 1,
            email.date,
            email.to.as_deref().unwrap_or(""),
            email.subject
        );
    }
}

pub(super) fn print_send_result(email: &Email) {
    println!(
        "sent {} -> {} | {}",
        email.from,
        email.to.as_deref().unwrap_or(""),
        email.subject
    );
}

pub(super) fn print_template(template: &str, output_path: Option<&Path>) {
    if let Some(path) = output_path {
        println!("wrote {}", path.display());
    } else {
        print!("{}", template);
    }
}

fn print_attachments(attachments: &[Attachment]) {
    if attachments.is_empty() {
        println!("attachments: 0");
        return;
    }

    println!("attachments:");
    for attachment in attachments {
        println!("  {}", format_attachment(attachment));
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
