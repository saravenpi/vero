mod args;
mod commands;

#[cfg(test)]
mod tests;

use anyhow::{anyhow, Context, Result};

pub(in crate::cli::parser) use args::{ensure_no_args, parse_u32, parse_usize, ArgCursor};
use commands::{parse_accounts, parse_draft, parse_inbox, parse_send, parse_sent};

use super::types::{CliCommand, CliInvocation, OutputFormat};

pub fn parse(raw_args: Vec<String>) -> Result<CliInvocation> {
    if raw_args.is_empty() {
        return Ok(CliInvocation {
            output: OutputFormat::Text,
            account: None,
            command: CliCommand::Tui,
        });
    }

    let mut args = ArgCursor::new(raw_args);
    let mut output = OutputFormat::Text;
    let mut account = None;

    loop {
        match args.peek() {
            Some("--json") => {
                args.next();
                output = OutputFormat::Json;
            }
            Some("--account") | Some("-a") => {
                let flag = args.next().context("Missing account flag")?;
                account = Some(args.value(&flag)?);
            }
            Some("help") | Some("-h") | Some("--help") => {
                args.next();
                return ensure_empty(
                    CliInvocation {
                        output,
                        account,
                        command: CliCommand::Help,
                    },
                    &args,
                );
            }
            Some("version") | Some("-v") | Some("--version") => {
                args.next();
                return ensure_empty(
                    CliInvocation {
                        output,
                        account,
                        command: CliCommand::Version,
                    },
                    &args,
                );
            }
            _ => break,
        }
    }

    if args.is_empty() {
        return Err(anyhow!("Missing command"));
    }

    let command = match args.next().as_deref() {
        Some("tui") => {
            ensure_no_args(&args)?;
            CliCommand::Tui
        }
        Some("accounts") => {
            parse_accounts(&mut args)?;
            CliCommand::AccountsList
        }
        Some("inbox") => CliCommand::Inbox(parse_inbox(&mut args)?),
        Some("sent") => CliCommand::Sent(parse_sent(&mut args)?),
        Some("send") => CliCommand::Send(parse_send(&mut args)?),
        Some("draft") => CliCommand::Draft(parse_draft(&mut args)?),
        Some(other) => return Err(anyhow!("Unknown command '{}'", other)),
        None => CliCommand::Tui,
    };

    Ok(CliInvocation {
        output,
        account,
        command,
    })
}

fn ensure_empty(invocation: CliInvocation, args: &ArgCursor) -> Result<CliInvocation> {
    ensure_no_args(args)?;
    Ok(invocation)
}
