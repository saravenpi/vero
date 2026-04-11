mod args;
mod commands;

#[cfg(test)]
mod tests;

use anyhow::{anyhow, Result};

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

    let (output, account, positional) = extract_global_flags(raw_args)?;
    let mut args = ArgCursor::new(positional);

    if args.is_empty() {
        return Ok(CliInvocation {
            output,
            account,
            command: CliCommand::Tui,
        });
    }

    match args.peek() {
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
        _ => {}
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

fn extract_global_flags(
    raw_args: Vec<String>,
) -> Result<(OutputFormat, Option<String>, Vec<String>)> {
    let mut output = OutputFormat::Text;
    let mut account = None;
    let mut positional = Vec::new();
    let mut iter = raw_args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--json" => {
                output = OutputFormat::Json;
            }
            "--account" | "-a" => {
                let value = iter
                    .next()
                    .ok_or_else(|| anyhow!("Missing value for {}", arg))?;
                account = Some(value);
            }
            _ => {
                positional.push(arg);
            }
        }
    }

    Ok((output, account, positional))
}
