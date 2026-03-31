use anyhow::Result;

use crate::cli::types::{
    CliCommand, CliInvocation, DraftCommand, InboxCommand, SendBody, SendCommand, SentCommand,
};
use crate::config::VeroConfig;
use crate::models::InboxFilter;
use crate::services;

use super::output;

pub async fn execute(config: VeroConfig, invocation: CliInvocation) -> Result<()> {
    let CliInvocation {
        output,
        account,
        command,
    } = invocation;

    match command {
        CliCommand::AccountsList => {
            let accounts = services::list_accounts(&config);
            output::print_accounts(output, &accounts)
        }
        CliCommand::Inbox(command) => {
            execute_inbox(&config, output, account.as_deref(), command).await
        }
        CliCommand::Sent(command) => {
            execute_sent(&config, output, account.as_deref(), command).await
        }
        CliCommand::Send(command) => {
            execute_send(&config, output, account.as_deref(), command).await
        }
        CliCommand::Draft(command) => execute_draft(output, command),
        CliCommand::Help | CliCommand::Version | CliCommand::Tui => Ok(()),
    }
}

async fn execute_inbox(
    config: &VeroConfig,
    output: crate::cli::types::OutputFormat,
    account_selector: Option<&str>,
    command: InboxCommand,
) -> Result<()> {
    let account = services::resolve_account(config, account_selector)?;

    match command {
        InboxCommand::List { filter, limit } => {
            let filter = filter.unwrap_or_else(|| InboxFilter::from_str(&config.inbox_view));
            let snapshot = services::load_inbox(&account, filter).await?;
            let crate::services::InboxSnapshot {
                emails,
                unseen_count,
            } = snapshot;
            let emails = apply_limit(emails, limit);
            output::print_inbox_list(output, filter, unseen_count, &emails)
        }
        InboxCommand::Show { uid } => {
            let email = services::read_inbox_email(&account, uid).await?;
            output::print_email(output, &email, None)
        }
        InboxCommand::Delete { uid } => {
            services::delete_inbox_email(&account, uid).await?;
            output::print_deleted(output, uid)
        }
        InboxCommand::UnreadCount => {
            let count = services::unread_count(&account).await?;
            output::print_unread_count(output, count)
        }
    }
}

async fn execute_sent(
    config: &VeroConfig,
    output: crate::cli::types::OutputFormat,
    account_selector: Option<&str>,
    command: SentCommand,
) -> Result<()> {
    let account = services::resolve_account(config, account_selector)?;

    match command {
        SentCommand::List { limit } => {
            let emails = apply_limit(services::load_sent_emails(&account)?, limit);
            output::print_sent(output, &emails)
        }
        SentCommand::Show { index } => {
            let email = services::read_sent_email(&account, index)?;
            output::print_sent_email(output, index, &email)
        }
    }
}

async fn execute_send(
    config: &VeroConfig,
    output: crate::cli::types::OutputFormat,
    account_selector: Option<&str>,
    command: SendCommand,
) -> Result<()> {
    let account = services::resolve_account(config, account_selector)?;

    let draft = match command {
        SendCommand::DraftFile { path } => services::parse_draft_input(&path)?.to_draft(),
        SendCommand::Fields {
            to,
            cc,
            bcc,
            subject,
            body,
            attachments,
        } => {
            let body = resolve_body(body)?;
            services::build_draft(to, cc, bcc, subject, body, attachments)?
        }
    };

    let email = services::send_draft(&account, draft).await?;
    output::print_send_result(output, &email)
}

fn execute_draft(output: crate::cli::types::OutputFormat, command: DraftCommand) -> Result<()> {
    match command {
        DraftCommand::Template { output_path } => {
            if let Some(path) = output_path.as_ref() {
                services::write_template(path)?;
                output::print_template(output, "", Some(path))
            } else {
                let template = services::create_template();
                output::print_template(output, &template, None)
            }
        }
    }
}

fn resolve_body(body: SendBody) -> Result<String> {
    match body {
        SendBody::Inline(body) => Ok(body),
        SendBody::File(path) => services::read_text_input(&path),
        SendBody::Empty => Ok(String::new()),
    }
}

fn apply_limit<T>(items: Vec<T>, limit: Option<usize>) -> Vec<T> {
    match limit {
        Some(limit) => items.into_iter().take(limit).collect(),
        None => items,
    }
}
