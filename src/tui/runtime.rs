use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use crate::config::VeroConfig;
use crate::models::Email;
use crate::services::{self, InboxSnapshot};
use crate::storage;
use crate::tui::app::{ComposeStep, Screen};
use crate::tui::{handlers, is_quit_key, render, App, AppEvent, EventHandler};

type InboxLoadTask = tokio::task::JoinHandle<Result<InboxSnapshot>>;
type SentLoadTask = tokio::task::JoinHandle<Result<Vec<Email>>>;

pub async fn run(config: VeroConfig) -> Result<()> {
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

    result
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

        maybe_spawn_inbox_load(&mut app, &mut inbox_load_task);
        maybe_spawn_sent_load(&mut app, &mut sent_load_task);
        handle_editor_open(&mut app)?;
        handle_inbox_load_result(&mut app, &mut inbox_load_task).await?;
        handle_sent_load_result(&mut app, &mut sent_load_task).await?;

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

                    handlers::handle_key_event(&mut app, key).await?;

                    if app.should_quit {
                        break;
                    }
                }
                AppEvent::Tick => app.tick_spinner(),
            }
        }
    }

    Ok(())
}

fn maybe_spawn_inbox_load(app: &mut App, task: &mut Option<InboxLoadTask>) {
    if !app.needs_inbox_load || task.is_some() {
        return;
    }

    app.needs_inbox_load = false;

    if let Some(account) = app.current_account.clone() {
        let filter = app.inbox_filter;
        *task = Some(tokio::spawn(async move {
            services::load_inbox(&account, filter).await
        }));
    }
}

fn maybe_spawn_sent_load(app: &mut App, task: &mut Option<SentLoadTask>) {
    if !app.needs_sent_load || task.is_some() {
        return;
    }

    app.needs_sent_load = false;

    if let Some(account) = app.current_account.clone() {
        *task = Some(tokio::spawn(
            async move { services::load_sent_emails(&account) },
        ));
    }
}

fn handle_editor_open(app: &mut App) -> Result<()> {
    if !app.needs_editor_open {
        return Ok(());
    }

    app.needs_editor_open = false;

    if app.config.editor.is_none() {
        app.compose_step = ComposeStep::NoEditor;
        return Ok(());
    }

    let Some(account) = app.current_account.clone() else {
        app.error_message = Some("No account selected".to_string());
        app.navigate_to(Screen::Inbox);
        return Ok(());
    };

    match storage::create_draft_file(&account.email) {
        Ok(draft_path) => {
            app.compose_draft_path = Some(draft_path.clone());

            match crate::tui::external::open_editor_for_draft(
                app.config.editor.as_ref().unwrap(),
                &draft_path,
            ) {
                Ok(()) => {
                    app.needs_full_redraw = true;
                    match services::parse_draft_input(&draft_path) {
                        Ok(parsed) => {
                            app.compose_draft = parsed.to_draft();
                            app.compose_step = ComposeStep::Preview;
                            app.error_message = None;
                        }
                        Err(error) => {
                            app.error_message = Some(format!("Draft parsing error: {}", error));
                            app.navigate_to(Screen::Inbox);
                            storage::delete_draft_file(&draft_path).ok();
                        }
                    }
                }
                Err(error) => {
                    app.needs_full_redraw = true;
                    app.error_message = Some(format!("Editor error: {}", error));
                    app.navigate_to(Screen::Inbox);
                    storage::delete_draft_file(&draft_path).ok();
                }
            }
        }
        Err(error) => {
            app.error_message = Some(format!("Failed to create draft: {}", error));
            app.navigate_to(Screen::Inbox);
        }
    }

    Ok(())
}

async fn handle_inbox_load_result(app: &mut App, task: &mut Option<InboxLoadTask>) -> Result<()> {
    let Some(load_task) = task.as_mut() else {
        return Ok(());
    };

    if !load_task.is_finished() {
        return Ok(());
    }

    let result = task.take().unwrap().await;

    match result {
        Ok(Ok(snapshot)) if !app.cancel_inbox_load => {
            let mut emails = snapshot.emails;
            merge_loaded_email_bodies(&app.inbox_emails, &mut emails);
            app.inbox_emails = emails;
            if app.inbox_selected >= app.inbox_emails.len() && !app.inbox_emails.is_empty() {
                app.inbox_selected = app.inbox_emails.len() - 1;
            }
            app.inbox_unseen_count = snapshot.unseen_count;
            app.inbox_loading = false;
            app.inbox_error = None;
        }
        Ok(Err(error)) if !app.cancel_inbox_load => {
            app.inbox_error = Some(format!("Failed to fetch emails: {}", error));
            app.inbox_loading = false;
        }
        Err(error) => {
            app.inbox_error = Some(format!("Task error: {}", error));
            app.inbox_loading = false;
        }
        _ => {}
    }

    app.cancel_inbox_load = false;

    Ok(())
}

async fn handle_sent_load_result(app: &mut App, task: &mut Option<SentLoadTask>) -> Result<()> {
    let Some(load_task) = task.as_mut() else {
        return Ok(());
    };

    if !load_task.is_finished() {
        return Ok(());
    }

    let result = task.take().unwrap().await;

    match result {
        Ok(Ok(emails)) if !app.cancel_sent_load => {
            app.sent_emails = emails;
            app.sent_loading = false;
            app.sent_error = None;
        }
        Ok(Err(error)) if !app.cancel_sent_load => {
            app.sent_error = Some(format!("Failed to load sent emails: {}", error));
            app.sent_loading = false;
        }
        Err(error) => {
            app.sent_error = Some(format!("Task error: {}", error));
            app.sent_loading = false;
        }
        _ => {}
    }

    app.cancel_sent_load = false;

    Ok(())
}

fn merge_loaded_email_bodies(existing_emails: &[Email], loaded_emails: &mut [Email]) {
    for email in loaded_emails {
        if let Some(existing) = existing_emails
            .iter()
            .find(|existing| existing.uid == email.uid)
        {
            if !existing.body.is_empty() {
                email.body = existing.body.clone();
                email.attachments = existing.attachments.clone();
            }
        }
    }
}
