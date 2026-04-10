use anyhow::Result;

use crate::models::Email;
use crate::services;
use crate::tui::App;

pub(in crate::tui::runtime) type SentLoadTask = tokio::task::JoinHandle<Result<Vec<Email>>>;

pub(in crate::tui::runtime) fn maybe_spawn_sent_load(
    app: &mut App,
    task: &mut Option<SentLoadTask>,
) {
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

pub(in crate::tui::runtime) async fn handle_sent_load_result(
    app: &mut App,
    task: &mut Option<SentLoadTask>,
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
        Ok(Ok(emails)) => {
            app.sent_emails = emails;
            app.clamp_sent_selection();
            app.sent_loading = false;
            app.sent_error = None;
        }
        Ok(Err(error)) => {
            app.set_sent_error(format!("Failed to load sent emails: {}", error));
            app.sent_loading = false;
        }
        Err(error) => {
            app.set_sent_error(format!("Task error: {}", error));
            app.sent_loading = false;
        }
    }

    Ok(())
}
