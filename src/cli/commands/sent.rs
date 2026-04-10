use anyhow::Result;

use crate::cli::output;
use crate::cli::types::{OutputFormat, SentCommand};
use crate::config::VeroConfig;
use crate::services;

use super::limit::apply_limit;

pub(super) async fn execute(
    config: &VeroConfig,
    output: OutputFormat,
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
