use anyhow::{anyhow, Result};

use crate::cli::types::SentCommand;

use super::super::{ensure_no_args, parse_usize, ArgCursor};

pub(in crate::cli::parser) fn parse_sent(args: &mut ArgCursor) -> Result<SentCommand> {
    match args.peek() {
        None => parse_sent_list(args),
        Some("list") => {
            args.next();
            parse_sent_list(args)
        }
        Some("show") => {
            args.next();
            let index = parse_usize(&args.value("show")?, "index")?;
            if index == 0 {
                return Err(anyhow!("Sent email index must be at least 1"));
            }
            ensure_no_args(args)?;
            Ok(SentCommand::Show { index })
        }
        Some(other) => Err(anyhow!("Unknown sent command '{}'", other)),
    }
}

fn parse_sent_list(args: &mut ArgCursor) -> Result<SentCommand> {
    let mut limit = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--limit" => {
                limit = Some(parse_usize(&args.value("--limit")?, "limit")?);
            }
            other => return Err(anyhow!("Unknown sent list option '{}'", other)),
        }
    }

    Ok(SentCommand::List { limit })
}
