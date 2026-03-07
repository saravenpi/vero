mod config;
mod email;
mod email_file;
mod models;
mod storage;
mod tui;

use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use config::VeroConfig;
use models::{Email, InboxFilter, ViewMode};
use tui::{is_quit_key, render, App, AppEvent, EventHandler};
use std::fs;
use std::io::Write;
use std::process::Command;

const VERSION: &str = "2.0.0";

type InboxLoadTask = tokio::task::JoinHandle<Result<(Vec<Email>, usize)>>;
type SentLoadTask = tokio::task::JoinHandle<Result<Vec<Email>>>;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "version" | "-v" | "--version" => {
                println!("Vero v{} (Rust)", VERSION);
                return Ok(());
            }
            "help" | "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ => {
                println!("Unknown command: {}", args[1]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    let config = VeroConfig::load().context("Failed to load config")?;

    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    let result = run_app(&mut terminal, config).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config: VeroConfig,
) -> Result<()> {
    let mut app = App::new(config);
    let mut events = EventHandler::new(Duration::from_millis(100));
    let mut inbox_load_task: Option<InboxLoadTask> = None;
    let mut sent_load_task: Option<SentLoadTask> = None;

    loop {
        if app.needs_full_redraw {
            terminal.clear()?;
            app.needs_full_redraw = false;
        }

        terminal.draw(|frame| render(frame, &app))?;

        if app.needs_inbox_load && inbox_load_task.is_none() {
            app.needs_inbox_load = false;
            if let Some(account) = app.current_account.clone() {
                let filter = app.inbox_filter;
                inbox_load_task = Some(tokio::spawn(async move {
                    let emails = email::fetch_emails(&account.imap, filter).await?;
                    let count = email::fetch_unseen_count(&account.imap).await.unwrap_or(0);
                    Ok((emails, count))
                }));
            }
        }

        if app.needs_sent_load && sent_load_task.is_none() {
            app.needs_sent_load = false;
            if let Some(account) = app.current_account.clone() {
                sent_load_task = Some(tokio::spawn(async move {
                    storage::load_sent_emails(&account.email)
                }));
            }
        }

        if app.needs_editor_open {
            app.needs_editor_open = false;

            if app.config.editor.is_none() {
                app.compose_step = tui::app::ComposeStep::NoEditor;
            } else {
                let account = app.current_account.as_ref().unwrap();
                match storage::create_draft_file(&account.email) {
                    Ok(draft_path) => {
                        app.compose_draft_path = Some(draft_path.clone());

                        match open_editor_for_draft(app.config.editor.as_ref().unwrap(), &draft_path) {
                            Ok(()) => {
                                app.needs_full_redraw = true;
                                match parse_draft_and_validate(&draft_path) {
                                    Ok(parsed) => {
                                        app.compose_draft = parsed.to_draft();
                                        app.compose_step = tui::app::ComposeStep::Preview;
                                        app.error_message = None;
                                    }
                                    Err(e) => {
                                        app.error_message = Some(format!("Draft parsing error: {}", e));
                                        app.navigate_to(tui::app::Screen::Inbox);
                                        storage::delete_draft_file(&draft_path).ok();
                                    }
                                }
                            }
                            Err(e) => {
                                app.needs_full_redraw = true;
                                app.error_message = Some(format!("Editor error: {}", e));
                                app.navigate_to(tui::app::Screen::Inbox);
                                storage::delete_draft_file(&draft_path).ok();
                            }
                        }
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Failed to create draft: {}", e));
                        app.navigate_to(tui::app::Screen::Inbox);
                    }
                }
            }
        }

        if let Some(task) = inbox_load_task.as_mut() {
            if task.is_finished() {
                let result = inbox_load_task.take().unwrap().await;
                match result {
                    Ok(Ok((mut emails, count))) => {
                        if !app.cancel_inbox_load {
                            for email in &mut emails {
                                if let Some(existing) = app.inbox_emails.iter().find(|e| e.uid == email.uid) {
                                    if !existing.body.is_empty() {
                                        email.body = existing.body.clone();
                                        email.attachments = existing.attachments.clone();
                                    }
                                }
                            }
                            app.inbox_emails = emails;
                            if app.inbox_selected >= app.inbox_emails.len() && !app.inbox_emails.is_empty() {
                                app.inbox_selected = app.inbox_emails.len() - 1;
                            }
                            app.inbox_unseen_count = count;
                            app.inbox_loading = false;
                            app.inbox_error = None;
                        }
                    }
                    Ok(Err(e)) => {
                        if !app.cancel_inbox_load {
                            app.inbox_error = Some(format!("Failed to fetch emails: {}", e));
                            app.inbox_loading = false;
                        }
                    }
                    Err(e) => {
                        app.inbox_error = Some(format!("Task error: {}", e));
                        app.inbox_loading = false;
                    }
                }
                app.cancel_inbox_load = false;
            }
        }

        if let Some(task) = sent_load_task.as_mut() {
            if task.is_finished() {
                let result = sent_load_task.take().unwrap().await;
                match result {
                    Ok(Ok(emails)) => {
                        if !app.cancel_sent_load {
                            app.sent_emails = emails;
                            app.sent_loading = false;
                            app.sent_error = None;
                        }
                    }
                    Ok(Err(e)) => {
                        if !app.cancel_sent_load {
                            app.sent_error = Some(format!("Failed to load sent emails: {}", e));
                            app.sent_loading = false;
                        }
                    }
                    Err(e) => {
                        app.sent_error = Some(format!("Task error: {}", e));
                        app.sent_loading = false;
                    }
                }
                app.cancel_sent_load = false;
            }
        }

        if app.tick_auto_refresh() {
            app.needs_inbox_load = true;
        }

        if let Some(event) = events.next().await {
            match event {
                AppEvent::Key(key) => {
                    if matches!(key.code, KeyCode::Char('c'))
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }

                    if is_quit_key(&key) {
                        break;
                    }

                    handle_key_event(&mut app, key).await?;

                    if app.should_quit {
                        break;
                    }
                }
                AppEvent::Tick => {
                    app.tick_spinner();
                }
            }
        }
    }

    Ok(())
}

async fn handle_key_event(app: &mut App, key: crossterm::event::KeyEvent) -> Result<()> {
    match app.screen {
        tui::app::Screen::AccountSelection => handle_account_selection_keys(app, key).await?,
        tui::app::Screen::Inbox => handle_inbox_keys(app, key).await?,
        tui::app::Screen::Sent => handle_sent_keys(app, key).await?,
        tui::app::Screen::Compose => handle_compose_keys(app, key).await?,
    }

    Ok(())
}

async fn handle_account_selection_keys(
    app: &mut App,
    key: crossterm::event::KeyEvent,
) -> Result<()> {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
        }
        KeyCode::Enter => {
            if app.account_selected < app.config.accounts.len() {
                app.current_account = Some(app.config.accounts[app.account_selected].clone());
                app.navigate_to(tui::app::Screen::Inbox);
            }
        }
        _ => {}
    }
    Ok(())
}


async fn handle_inbox_keys(app: &mut App, key: crossterm::event::KeyEvent) -> Result<()> {
    use tui::app::FocusedElement;

    if app.focused == FocusedElement::MenuBar {
        match key.code {
            KeyCode::Esc => {
                if app.config.accounts.len() > 1 {
                    app.navigate_to(tui::app::Screen::AccountSelection);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.menu_next();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.menu_previous();
            }
            KeyCode::Enter => {
                app.menu_select();
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.toggle_focus();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.inbox_view_mode == ViewMode::Detail {
        match key.code {
            KeyCode::Esc => {
                app.inbox_view_mode = ViewMode::List;
                app.inbox_scroll_offset = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.inbox_scroll_offset = app.inbox_scroll_offset.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.inbox_scroll_offset = app.inbox_scroll_offset.saturating_sub(1);
            }
            KeyCode::PageDown => {
                app.inbox_scroll_offset = app.inbox_scroll_offset.saturating_add(10);
            }
            KeyCode::PageUp => {
                app.inbox_scroll_offset = app.inbox_scroll_offset.saturating_sub(10);
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.toggle_focus();
            }
            KeyCode::Char('e') => {
                if !app.inbox_emails.is_empty() && app.inbox_selected < app.inbox_emails.len() {
                    let editor = app.config.editor.as_ref().or(app.config.viewer.as_ref());
                    if let Some(viewer) = editor {
                        let email = &app.inbox_emails[app.inbox_selected];
                        open_email_in_viewer(viewer, email)?;
                        app.needs_full_redraw = true;
                    }
                }
            }
            _ => {}
        }
        return Ok(());
    }

    match key.code {
        KeyCode::Esc => {
            app.navigate_to(tui::app::Screen::Inbox);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
        }
        KeyCode::Enter => {
            if !app.inbox_emails.is_empty() && app.inbox_selected < app.inbox_emails.len() {
                let account = app.current_account.as_ref().unwrap().clone();
                let uid = app.inbox_emails[app.inbox_selected].uid;

                let (body, attachments) = email::fetch_email_body(&account.imap, uid).await?;

                app.inbox_emails[app.inbox_selected].body = body;
                app.inbox_emails[app.inbox_selected].attachments = attachments;

                let email = app.inbox_emails[app.inbox_selected].clone();
                storage::save_seen_email(&account.email, email)?;

                app.inbox_scroll_offset = 0;
                app.inbox_view_mode = ViewMode::Detail;
            }
        }
        KeyCode::Char('d') => {
            if !app.inbox_emails.is_empty() && app.inbox_selected < app.inbox_emails.len() {
                let email = app.inbox_emails[app.inbox_selected].clone();
                let account = app.current_account.as_ref().unwrap().clone();

                email::delete_email(&account.imap, email.uid).await?;
                storage::delete_seen_email(&account.email, &email)?;

                app.inbox_emails.remove(app.inbox_selected);
                if app.inbox_selected > 0 && app.inbox_selected >= app.inbox_emails.len() {
                    app.inbox_selected -= 1;
                }

                app.status_message = Some("Email deleted".to_string());
            }
        }
        KeyCode::Char('u') => {
            app.inbox_filter = InboxFilter::Unseen;
            app.needs_inbox_load = true;
            app.inbox_loading = true;
            app.inbox_error = None;
        }
        KeyCode::Char('s') => {
            app.inbox_filter = InboxFilter::Seen;
            app.needs_inbox_load = true;
            app.inbox_loading = true;
            app.inbox_error = None;
        }
        KeyCode::Char('a') => {
            app.inbox_filter = InboxFilter::All;
            app.needs_inbox_load = true;
            app.inbox_loading = true;
            app.inbox_error = None;
        }
        KeyCode::Char('r') => {
            app.cancel_inbox_load = false;
            app.needs_inbox_load = true;
            app.inbox_loading = true;
            app.inbox_error = None;
        }
        KeyCode::Tab | KeyCode::BackTab => {
            app.toggle_focus();
        }
        _ => {}
    }
    Ok(())
}

async fn handle_sent_keys(app: &mut App, key: crossterm::event::KeyEvent) -> Result<()> {
    use tui::app::FocusedElement;

    if app.focused == FocusedElement::MenuBar {
        match key.code {
            KeyCode::Esc => {
                if app.config.accounts.len() > 1 {
                    app.navigate_to(tui::app::Screen::AccountSelection);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.menu_next();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.menu_previous();
            }
            KeyCode::Enter => {
                app.menu_select();
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.toggle_focus();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.sent_view_mode == ViewMode::Detail {
        match key.code {
            KeyCode::Esc => {
                app.sent_view_mode = ViewMode::List;
                app.sent_scroll_offset = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.sent_scroll_offset = app.sent_scroll_offset.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.sent_scroll_offset = app.sent_scroll_offset.saturating_sub(1);
            }
            KeyCode::PageDown => {
                app.sent_scroll_offset = app.sent_scroll_offset.saturating_add(10);
            }
            KeyCode::PageUp => {
                app.sent_scroll_offset = app.sent_scroll_offset.saturating_sub(10);
            }
            KeyCode::Tab | KeyCode::BackTab => {
                app.toggle_focus();
            }
            KeyCode::Char('e') => {
                if !app.sent_emails.is_empty() && app.sent_selected < app.sent_emails.len() {
                    let editor = app.config.editor.as_ref().or(app.config.viewer.as_ref());
                    if let Some(viewer) = editor {
                        let email = &app.sent_emails[app.sent_selected];
                        open_email_in_viewer(viewer, email)?;
                        app.needs_full_redraw = true;
                    }
                }
            }
            _ => {}
        }
        return Ok(());
    }

    match key.code {
        KeyCode::Esc => {
            app.navigate_to(tui::app::Screen::Inbox);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
        }
        KeyCode::Enter => {
            if !app.sent_emails.is_empty() && app.sent_selected < app.sent_emails.len() {
                app.sent_scroll_offset = 0;
                app.sent_view_mode = ViewMode::Detail;
            }
        }
        KeyCode::Char('r') => {
            app.cancel_sent_load = false;
            app.needs_sent_load = true;
            app.sent_loading = true;
            app.sent_error = None;
        }
        KeyCode::Tab | KeyCode::BackTab => {
            app.toggle_focus();
        }
        _ => {}
    }
    Ok(())
}

async fn handle_compose_keys(app: &mut App, key: crossterm::event::KeyEvent) -> Result<()> {
    use tui::app::ComposeStep;

    match app.compose_step {
        ComposeStep::NoEditor => {
            app.navigate_to(tui::app::Screen::Inbox);
            return Ok(());
        }
        ComposeStep::Editing => {
            return Ok(());
        }
        ComposeStep::Preview => {
            match key.code {
                KeyCode::Enter => {
                    let account = app.current_account.as_ref().unwrap();
                    let draft = app.compose_draft.clone();

                    email::send_email(&account.smtp, &account.email, draft.clone()).await?;

                    let email = models::Email {
                        from: account.email.clone(),
                        to: Some(draft.to.clone()),
                        cc: if draft.cc.is_empty() {
                            None
                        } else {
                            Some(draft.cc.clone())
                        },
                        bcc: if draft.bcc.is_empty() {
                            None
                        } else {
                            Some(draft.bcc.clone())
                        },
                        subject: draft.subject.clone(),
                        date: chrono::Utc::now().to_rfc2822(),
                        body: draft.body.clone(),
                        timestamp: chrono::Utc::now(),
                        attachments: draft.attachments.clone(),
                        uid: 0,
                    };

                    storage::save_sent_email(&account.email, email)?;

                    if let Some(ref draft_path) = app.compose_draft_path {
                        storage::delete_draft_file(draft_path).ok();
                    }

                    app.status_message = Some("Email sent successfully!".to_string());
                    app.navigate_to(tui::app::Screen::Inbox);
                }
                KeyCode::Char('e') => {
                    if let Some(ref draft_path) = app.compose_draft_path {
                        match open_editor_for_draft(app.config.editor.as_ref().unwrap(), draft_path)
                        {
                            Ok(()) => {
                                app.needs_full_redraw = true;
                                match parse_draft_and_validate(draft_path) {
                                    Ok(parsed) => {
                                        app.compose_draft = parsed.to_draft();
                                        app.error_message = None;
                                    }
                                    Err(e) => {
                                        app.error_message =
                                            Some(format!("Draft parsing error: {}", e));
                                    }
                                }
                            }
                            Err(e) => {
                                app.needs_full_redraw = true;
                                app.error_message = Some(format!("Editor error: {}", e));
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    if let Some(ref draft_path) = app.compose_draft_path {
                        storage::delete_draft_file(draft_path).ok();
                    }
                    app.navigate_to(tui::app::Screen::Inbox);
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn open_editor_for_draft(editor: &str, draft_path: &std::path::Path) -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    let editor_args: Vec<&str> = editor.split_whitespace().collect();
    let (editor_cmd, args) = editor_args
        .split_first()
        .context("Editor command is empty")?;

    let mut final_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    final_args.push(draft_path.to_string_lossy().to_string());

    let status = Command::new(editor_cmd)
        .args(&final_args)
        .status()
        .context("Failed to execute editor")?;

    enable_raw_mode()?;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    Ok(())
}

fn parse_draft_and_validate(
    draft_path: &std::path::PathBuf,
) -> Result<email_file::ParsedEmail> {
    let content = fs::read_to_string(draft_path).context("Failed to read draft file")?;

    let parsed = email_file::parse_email_file(&content)?;

    email_file::validate_attachments(&parsed.attachment_paths)?;

    Ok(parsed)
}

fn open_email_in_viewer(viewer: &str, email: &Email) -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("vero_email_{}.txt", chrono::Utc::now().timestamp()));

    let unknown = String::from("Unknown");
    let empty = String::new();
    let to = email.to.as_ref().unwrap_or(&unknown);
    let cc = email.cc.as_ref().unwrap_or(&empty);

    let content = format!(
        "From: {}\nTo: {}\n{}\nSubject: {}\nDate: {}\n\n{}\n",
        email.from,
        to,
        if !cc.is_empty() { format!("CC: {}\n", cc) } else { String::new() },
        email.subject,
        email.date,
        email.body
    );

    let mut file = fs::File::create(&temp_file)
        .context("Failed to create temporary file")?;
    file.write_all(content.as_bytes())
        .context("Failed to write email to temporary file")?;
    drop(file);

    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    let viewer_args: Vec<&str> = viewer.split_whitespace().collect();
    let (viewer_cmd, args) = viewer_args.split_first()
        .context("Viewer command is empty")?;

    let mut final_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    if viewer_cmd.contains("vim") || viewer_cmd.contains("nvim") {
        final_args.push("-R".to_string());
    }

    final_args.push(temp_file.to_string_lossy().to_string());

    let status = Command::new(viewer_cmd)
        .args(&final_args)
        .status()
        .context("Failed to execute viewer")?;

    enable_raw_mode()?;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    fs::remove_file(&temp_file).ok();

    if !status.success() {
        anyhow::bail!("Viewer exited with non-zero status");
    }

    Ok(())
}

fn print_help() {
    let help = r#"Vero - Terminal Email Client

Usage:
  vero              Start the email client
  vero version      Show version information
  vero help         Show this help message

Configuration:
  Create a ~/.vero.yml file with your accounts:

  accounts:
    - email: your@email.com
      imap:
        user: your@email.com
        password: your-password
        host: imap.example.com
        port: 993
      smtp:
        user: your@email.com
        password: your-password
        host: smtp.example.com
        port: 465

  # Optional: Viewer for opening emails in read-only mode
  viewer: nvim           # Use 'nvim', 'vim', 'less', or any text viewer

  # Optional: Auto-refresh inbox (in seconds, default: 0 = disabled)
  auto_refresh: 30

  # Optional: Default inbox view (unseen, seen, or all)
  inbox_view: all

Navigation:
  ↑/↓ or j/k        Navigate lists
  Enter             Select/Open
  ESC               Go back
  q                 Quit
  Tab               Toggle focus

Inbox:
  u                 Show unseen emails only
  s                 Show seen emails only
  a                 Show all emails
  r                 Refresh
  d                 Delete email
  e                 Open email in viewer (detail view)

Compose:
  Tab/Enter         Next field
  Ctrl+D            Preview and send
  ESC               Cancel
"#;
    print!("{}", help);
}
