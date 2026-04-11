use super::*;
use crate::models::InboxFilter;

use super::super::types::{InboxCommand, SendBody, SendCommand};

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
fn account_flag_is_accepted_after_subcommand() {
    let cli = parse(vec![
        "inbox".to_string(),
        "unread-count".to_string(),
        "--account".to_string(),
        "user@example.com".to_string(),
    ])
    .unwrap();

    assert_eq!(cli.account.as_deref(), Some("user@example.com"));
    assert!(matches!(
        cli.command,
        CliCommand::Inbox(InboxCommand::UnreadCount)
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
