use anyhow::Result;

use crate::cli::output;
use crate::cli::types::{InboxCommand, OutputFormat};
use crate::config::VeroConfig;
use crate::models::InboxFilter;
use crate::services;

use super::limit::apply_limit;

pub(super) async fn execute(
    config: &VeroConfig,
    output: OutputFormat,
    account_selector: Option<&str>,
    command: InboxCommand,
) -> Result<()> {
    let account = services::resolve_account(config, account_selector)?;

    match command {
        InboxCommand::List { filter, limit } => {
            let filter = filter.unwrap_or_else(|| InboxFilter::from_str(&config.inbox_view));
            let snapshot = services::load_inbox(&account).await?;
            let unseen_count = snapshot.unseen_count;
            let emails = apply_limit(snapshot.filtered_emails(filter), limit);
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
        InboxCommand::Download { uid, index } => {
            let folder = config
                .download_folder
                .as_deref()
                .unwrap_or("~/Downloads");
            let paths =
                services::download_inbox_attachments(&account, uid, index, folder).await?;
            output::print_download_result(output, &paths)
        }
        InboxCommand::UnreadCount => {
            let count = services::unread_count(&account).await?;
            output::print_unread_count(output, count)
        }
    }
}
