use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::models::{Email, InboxFilter};
use crate::services::AccountSummary;

pub(super) fn print_accounts(accounts: &[AccountSummary]) -> Result<()> {
    print_json(&AccountsResponse { accounts })
}

pub(super) fn print_inbox_list(
    filter: InboxFilter,
    unread_count: usize,
    emails: &[Email],
) -> Result<()> {
    print_json(&InboxListResponse {
        filter: filter.as_str(),
        unread_count,
        emails,
    })
}

pub(super) fn print_email(email: &Email, index: Option<usize>) -> Result<()> {
    print_json(&EmailResponse { index, email })
}

pub(super) fn print_download_result(paths: &[PathBuf]) -> Result<()> {
    let saved: Vec<_> = paths.iter().map(|p| p.display().to_string()).collect();
    print_json(&DownloadResponse { saved })
}

pub(super) fn print_unread_count(unread_count: usize) -> Result<()> {
    print_json(&UnreadCountResponse { unread_count })
}

pub(super) fn print_deleted(uid: u32) -> Result<()> {
    print_json(&DeleteResponse { deleted_uid: uid })
}

pub(super) fn print_sent(emails: &[Email]) -> Result<()> {
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

pub(super) fn print_send_result(email: &Email) -> Result<()> {
    print_json(&SendResponse { email })
}

pub(super) fn print_template(template: &str, output_path: Option<&Path>) -> Result<()> {
    print_json(&TemplateResponse {
        output_path: output_path.map(|path| path.display().to_string()),
        template: output_path.is_none().then_some(template.to_string()),
    })
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
struct DownloadResponse {
    saved: Vec<String>,
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
