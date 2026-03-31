pub fn print_help() {
    print!(
        r#"Vero - terminal email client

Usage:
  vero
  vero [--account <email-or-index>] [--json] <command>

Commands:
  tui                               Start the TUI
  accounts [list]                   List configured accounts
  inbox [list] [--filter <name>]    List inbox emails
  inbox show <uid>                  Read an inbox email
  inbox delete <uid>                Delete an inbox email
  inbox unread-count                Show unread count
  sent [list] [--limit <n>]         List locally stored sent emails
  sent show <index>                 Show a sent email
  send --to <addr> --subject <subj> Send an email from flags
  send --draft <path|->             Send an email from a draft file or stdin
  draft [template] [--output <p>]   Print or write a draft template
  help                              Show this help
  version                           Show version information

Global options:
  --account, -a   Account email or 1-based index
  --json          Print command output as JSON

Examples:
  vero accounts
  vero --account work inbox list --filter unseen
  vero --account 1 inbox show 4242
  vero --account work send --to alice@example.com --subject "Ping" --body "Hi"
  vero --account work send --draft draft.eml
  vero draft template --output draft.eml
"#
    );
}
