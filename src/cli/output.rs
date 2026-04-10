mod responses;
mod text;

use anyhow::Result;
use std::path::Path;

use crate::cli::types::OutputFormat;
use crate::models::{Email, InboxFilter};
use crate::services::AccountSummary;

pub fn print_accounts(output: OutputFormat, accounts: &[AccountSummary]) -> Result<()> {
    match output {
        OutputFormat::Text => {
            text::print_accounts(accounts);
            Ok(())
        }
        OutputFormat::Json => responses::print_accounts(accounts),
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
            text::print_inbox_list(filter, unseen_count, emails);
            Ok(())
        }
        OutputFormat::Json => responses::print_inbox_list(filter, unseen_count, emails),
    }
}

pub fn print_email(output: OutputFormat, email: &Email, index: Option<usize>) -> Result<()> {
    match output {
        OutputFormat::Text => {
            text::print_email(email, index);
            Ok(())
        }
        OutputFormat::Json => responses::print_email(email, index),
    }
}

pub fn print_unread_count(output: OutputFormat, unread_count: usize) -> Result<()> {
    match output {
        OutputFormat::Text => {
            text::print_unread_count(unread_count);
            Ok(())
        }
        OutputFormat::Json => responses::print_unread_count(unread_count),
    }
}

pub fn print_download_result(output: OutputFormat, paths: &[std::path::PathBuf]) -> Result<()> {
    match output {
        OutputFormat::Text => {
            text::print_download_result(paths);
            Ok(())
        }
        OutputFormat::Json => responses::print_download_result(paths),
    }
}

pub fn print_deleted(output: OutputFormat, uid: u32) -> Result<()> {
    match output {
        OutputFormat::Text => {
            text::print_deleted(uid);
            Ok(())
        }
        OutputFormat::Json => responses::print_deleted(uid),
    }
}

pub fn print_sent(output: OutputFormat, emails: &[Email]) -> Result<()> {
    match output {
        OutputFormat::Text => {
            text::print_sent(emails);
            Ok(())
        }
        OutputFormat::Json => responses::print_sent(emails),
    }
}

pub fn print_sent_email(output: OutputFormat, index: usize, email: &Email) -> Result<()> {
    print_email(output, email, Some(index))
}

pub fn print_send_result(output: OutputFormat, email: &Email) -> Result<()> {
    match output {
        OutputFormat::Text => {
            text::print_send_result(email);
            Ok(())
        }
        OutputFormat::Json => responses::print_send_result(email),
    }
}

pub fn print_template(
    output: OutputFormat,
    template: &str,
    output_path: Option<&Path>,
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            text::print_template(template, output_path);
            Ok(())
        }
        OutputFormat::Json => responses::print_template(template, output_path),
    }
}
