use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

use crate::cli::types::{SendBody, SendCommand};

use super::super::ArgCursor;

pub(in crate::cli::parser) fn parse_send(args: &mut ArgCursor) -> Result<SendCommand> {
    let mut draft_path = None;
    let mut to = None;
    let mut cc = None;
    let mut bcc = None;
    let mut subject = None;
    let mut inline_body = None;
    let mut body_file = None;
    let mut attachments = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--draft" => {
                draft_path = Some(PathBuf::from(args.value("--draft")?));
            }
            "--to" => {
                to = Some(args.value("--to")?);
            }
            "--cc" => {
                cc = Some(args.value("--cc")?);
            }
            "--bcc" => {
                bcc = Some(args.value("--bcc")?);
            }
            "--subject" => {
                subject = Some(args.value("--subject")?);
            }
            "--body" => {
                inline_body = Some(args.value("--body")?);
            }
            "--body-file" => {
                body_file = Some(PathBuf::from(args.value("--body-file")?));
            }
            "--attach" | "--attachment" => {
                attachments.push(args.value(arg.as_str())?);
            }
            other => return Err(anyhow!("Unknown send option '{}'", other)),
        }
    }

    if let Some(path) = draft_path {
        if to.is_some()
            || cc.is_some()
            || bcc.is_some()
            || subject.is_some()
            || inline_body.is_some()
            || body_file.is_some()
            || !attachments.is_empty()
        {
            return Err(anyhow!(
                "--draft cannot be combined with other send options"
            ));
        }

        return Ok(SendCommand::DraftFile { path });
    }

    if inline_body.is_some() && body_file.is_some() {
        return Err(anyhow!("Use either --body or --body-file, not both"));
    }

    let body = if let Some(body) = inline_body {
        SendBody::Inline(body)
    } else if let Some(path) = body_file {
        SendBody::File(path)
    } else {
        SendBody::Empty
    };

    Ok(SendCommand::Fields {
        to: to.context("Missing required --to")?,
        cc,
        bcc,
        subject: subject.context("Missing required --subject")?,
        body,
        attachments,
    })
}
