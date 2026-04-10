use anyhow::{anyhow, Result};

use crate::cli::types::InboxCommand;
use crate::models::InboxFilter;

use super::super::{ensure_no_args, parse_u32, parse_usize, ArgCursor};

pub(in crate::cli::parser) fn parse_inbox(args: &mut ArgCursor) -> Result<InboxCommand> {
    match args.peek() {
        None => parse_inbox_list(args),
        Some("list") => {
            args.next();
            parse_inbox_list(args)
        }
        Some("show") => {
            args.next();
            let uid = parse_u32(&args.value("show")?, "uid")?;
            ensure_no_args(args)?;
            Ok(InboxCommand::Show { uid })
        }
        Some("delete") => {
            args.next();
            let uid = parse_u32(&args.value("delete")?, "uid")?;
            ensure_no_args(args)?;
            Ok(InboxCommand::Delete { uid })
        }
        Some("unread-count") | Some("count") => {
            args.next();
            ensure_no_args(args)?;
            Ok(InboxCommand::UnreadCount)
        }
        Some(other) => Err(anyhow!("Unknown inbox command '{}'", other)),
    }
}

fn parse_inbox_list(args: &mut ArgCursor) -> Result<InboxCommand> {
    let mut filter = None;
    let mut limit = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--filter" => {
                filter = Some(parse_filter(&args.value("--filter")?)?);
            }
            "--limit" => {
                limit = Some(parse_usize(&args.value("--limit")?, "limit")?);
            }
            other => return Err(anyhow!("Unknown inbox list option '{}'", other)),
        }
    }

    Ok(InboxCommand::List { filter, limit })
}

fn parse_filter(value: &str) -> Result<InboxFilter> {
    match value {
        "unseen" => Ok(InboxFilter::Unseen),
        "seen" => Ok(InboxFilter::Seen),
        "all" => Ok(InboxFilter::All),
        _ => Err(anyhow!(
            "Invalid inbox filter '{}'. Use unseen, seen, or all.",
            value
        )),
    }
}
