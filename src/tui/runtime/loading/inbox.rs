use anyhow::Result;

use crate::services::{self, InboxSnapshot};
use crate::tui::App;

pub(in crate::tui::runtime) type InboxLoadTask = tokio::task::JoinHandle<Result<InboxSnapshot>>;

pub(in crate::tui::runtime) fn maybe_load_cached_inbox(app: &mut App) {
    if !app.needs_inbox_cache_load {
        return;
    }

    app.needs_inbox_cache_load = false;

    let Some(account) = app.current_account.as_ref() else {
        return;
    };

    match services::load_cached_inbox(account) {
        Ok(snapshot) => {
            app.apply_inbox_snapshot(snapshot);
            app.inbox_error = None;
        }
        Err(error) => {
            if app.inbox_emails.is_empty() {
                app.set_inbox_error(format!("Failed to load cached inbox: {}", error));
            }
        }
    }
}

pub(in crate::tui::runtime) fn maybe_spawn_inbox_load(
    app: &mut App,
    task: &mut Option<InboxLoadTask>,
) {
    if !app.needs_inbox_load || task.is_some() {
        return;
    }

    app.needs_inbox_load = false;

    if let Some(account) = app.current_account.clone() {
        *task = Some(tokio::spawn(
            async move { services::load_inbox(&account).await },
        ));
    }
}

pub(in crate::tui::runtime) async fn handle_inbox_load_result(
    app: &mut App,
    task: &mut Option<InboxLoadTask>,
) -> Result<()> {
    let Some(load_task) = task.as_mut() else {
        return Ok(());
    };

    if !load_task.is_finished() {
        return Ok(());
    }

    let Some(load_task) = task.take() else {
        return Ok(());
    };

    match load_task.await {
        Ok(Ok(snapshot)) => {
            app.apply_inbox_snapshot(snapshot);
            app.inbox_loading = false;
            app.inbox_error = None;
        }
        Ok(Err(error)) => {
            app.set_inbox_error(format!("Failed to fetch emails: {}", error));
            app.inbox_loading = false;
        }
        Err(error) => {
            app.set_inbox_error(format!("Task error: {}", error));
            app.inbox_loading = false;
        }
    }

    Ok(())
}
