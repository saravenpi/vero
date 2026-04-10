mod draft;
mod inbox;
mod limit;
mod send;
mod sent;

use anyhow::Result;

use crate::cli::types::{CliCommand, CliInvocation};
use crate::config::VeroConfig;
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
            inbox::execute(&config, output, account.as_deref(), command).await
        }
        CliCommand::Sent(command) => {
            sent::execute(&config, output, account.as_deref(), command).await
        }
        CliCommand::Send(command) => {
            send::execute(&config, output, account.as_deref(), command).await
        }
        CliCommand::Draft(command) => draft::execute(output, command),
        CliCommand::Help | CliCommand::Version | CliCommand::Tui => Ok(()),
    }
}
