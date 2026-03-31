use anyhow::{anyhow, Context, Result};
use std::collections::VecDeque;
use std::path::PathBuf;

use crate::models::InboxFilter;

use super::types::{
    CliCommand, CliInvocation, DraftCommand, InboxCommand, OutputFormat, SendBody, SendCommand,
    SentCommand,
};

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
                let flag = args.next().unwrap();
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

fn parse_accounts(args: &mut ArgCursor) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }

    match args.next().as_deref() {
        Some("list") => ensure_no_args(args),
        Some(other) => Err(anyhow!("Unknown accounts command '{}'", other)),
        None => Ok(()),
    }
}

fn parse_inbox(args: &mut ArgCursor) -> Result<InboxCommand> {
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

fn parse_sent(args: &mut ArgCursor) -> Result<SentCommand> {
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

fn parse_send(args: &mut ArgCursor) -> Result<SendCommand> {
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

    let body = match (inline_body, body_file) {
        (Some(body), None) => SendBody::Inline(body),
        (None, Some(path)) => SendBody::File(path),
        (None, None) => SendBody::Empty,
        (Some(_), Some(_)) => unreachable!(),
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

fn parse_draft(args: &mut ArgCursor) -> Result<DraftCommand> {
    match args.peek() {
        None => Ok(DraftCommand::Template { output_path: None }),
        Some("template") => {
            args.next();
            let mut output_path = None;

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--output" => {
                        output_path = Some(PathBuf::from(args.value("--output")?));
                    }
                    other => return Err(anyhow!("Unknown draft option '{}'", other)),
                }
            }

            Ok(DraftCommand::Template { output_path })
        }
        Some(other) => Err(anyhow!("Unknown draft command '{}'", other)),
    }
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

fn parse_u32(value: &str, label: &str) -> Result<u32> {
    value
        .parse::<u32>()
        .with_context(|| format!("Invalid {} '{}'", label, value))
}

fn parse_usize(value: &str, label: &str) -> Result<usize> {
    value
        .parse::<usize>()
        .with_context(|| format!("Invalid {} '{}'", label, value))
}

fn ensure_empty(invocation: CliInvocation, args: &ArgCursor) -> Result<CliInvocation> {
    ensure_no_args(args)?;
    Ok(invocation)
}

fn ensure_no_args(args: &ArgCursor) -> Result<()> {
    if let Some(arg) = args.peek() {
        return Err(anyhow!("Unexpected argument '{}'", arg));
    }

    Ok(())
}

struct ArgCursor {
    args: VecDeque<String>,
}

impl ArgCursor {
    fn new(args: Vec<String>) -> Self {
        Self { args: args.into() }
    }

    fn peek(&self) -> Option<&str> {
        self.args.front().map(String::as_str)
    }

    fn next(&mut self) -> Option<String> {
        self.args.pop_front()
    }

    fn value(&mut self, flag: &str) -> Result<String> {
        self.next()
            .ok_or_else(|| anyhow!("Missing value for {}", flag))
    }

    fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tui_when_no_args() {
        let cli = parse(vec![]).unwrap();
        assert!(matches!(cli.command, CliCommand::Tui));
    }

    #[test]
    fn parses_global_flags_and_inbox_command() {
        let cli = parse(vec![
            "--json".to_string(),
            "--account".to_string(),
            "2".to_string(),
            "inbox".to_string(),
            "list".to_string(),
            "--filter".to_string(),
            "unseen".to_string(),
        ])
        .unwrap();

        assert!(matches!(cli.output, OutputFormat::Json));
        assert_eq!(cli.account.as_deref(), Some("2"));
        assert!(matches!(
            cli.command,
            CliCommand::Inbox(InboxCommand::List {
                filter: Some(InboxFilter::Unseen),
                limit: None
            })
        ));
    }

    #[test]
    fn parses_send_fields() {
        let cli = parse(vec![
            "send".to_string(),
            "--to".to_string(),
            "alice@example.com".to_string(),
            "--subject".to_string(),
            "hello".to_string(),
            "--body".to_string(),
            "hi".to_string(),
            "--attach".to_string(),
            "/tmp/file.txt".to_string(),
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            CliCommand::Send(SendCommand::Fields {
                ref to,
                ref subject,
                body: SendBody::Inline(ref body),
                ref attachments,
                ..
            }) if to == "alice@example.com"
                && subject == "hello"
                && body == "hi"
                && attachments == &vec!["/tmp/file.txt".to_string()]
        ));
    }

    #[test]
    fn rejects_mixed_send_sources() {
        let error = parse(vec![
            "send".to_string(),
            "--draft".to_string(),
            "mail.eml".to_string(),
            "--to".to_string(),
            "alice@example.com".to_string(),
        ])
        .unwrap_err();

        assert!(error.to_string().contains("--draft cannot be combined"));
    }
}
