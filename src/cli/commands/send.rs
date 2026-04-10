use anyhow::Result;

use crate::cli::output;
use crate::cli::types::{OutputFormat, SendBody, SendCommand};
use crate::config::VeroConfig;
use crate::services;

pub(super) async fn execute(
    config: &VeroConfig,
    output: OutputFormat,
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

fn resolve_body(body: SendBody) -> Result<String> {
    match body {
        SendBody::Inline(body) => Ok(body),
        SendBody::File(path) => services::read_text_input(&path),
        SendBody::Empty => Ok(String::new()),
    }
}
