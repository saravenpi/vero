mod editor;
mod loading;

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
use crate::tui::{handlers, is_quit_key, render, App, AppEvent, EventHandler};

use editor::handle_editor_open;
use loading::{
    handle_attachment_download_result, handle_drafts_load, handle_inbox_load_result,
    handle_send_result, handle_sent_load_result, handle_signature_load, maybe_load_cached_inbox,
    maybe_spawn_attachment_download, maybe_spawn_inbox_load, maybe_spawn_send,
    maybe_spawn_sent_load, AttachmentDownloadTask, ComposeSendTask, InboxLoadTask, SentLoadTask,
};

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
    let mut compose_send_task: Option<ComposeSendTask> = None;
    let mut attachment_download_task: Option<AttachmentDownloadTask> = None;

    loop {
        if app.needs_full_redraw {
            terminal.clear()?;
            app.needs_full_redraw = false;
        }

        maybe_load_cached_inbox(&mut app);
        terminal.draw(|frame| render(frame, &mut app))?;

        if app.cancel_inbox_load {
            app.cancel_inbox_load = false;
            if let Some(task) = inbox_load_task.take() {
                task.abort();
            }
        }
        maybe_spawn_inbox_load(&mut app, &mut inbox_load_task);
        maybe_spawn_sent_load(&mut app, &mut sent_load_task);
        maybe_spawn_send(&mut app, &mut compose_send_task);
        maybe_spawn_attachment_download(&mut app, &mut attachment_download_task);
        handle_drafts_load(&mut app);
        handle_signature_load(&mut app);
        handle_editor_open(&mut app)?;
        handle_inbox_load_result(&mut app, &mut inbox_load_task).await?;
        handle_sent_load_result(&mut app, &mut sent_load_task).await?;
        handle_send_result(&mut app, &mut compose_send_task).await?;
        handle_attachment_download_result(&mut app, &mut attachment_download_task).await?;

        if app.tick_auto_refresh() {
            app.needs_inbox_load = true;
            app.inbox_loading = true;
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
                }
                AppEvent::Tick => app.tick_spinner(),
            }
        }
    }

    Ok(())
}
