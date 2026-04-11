use anyhow::Result;
use std::path::PathBuf;

use crate::services;
use crate::storage;
use crate::tui::app::{App, Screen};
use crate::tui::error_messages::format_send_error;

pub(in crate::tui::runtime) type ComposeSendTask = tokio::task::JoinHandle<Result<Option<PathBuf>>>;

pub(in crate::tui::runtime) fn maybe_spawn_send(app: &mut App, task: &mut Option<ComposeSendTask>) {
    if !app.needs_email_send || task.is_some() {
        return;
    }

    app.needs_email_send = false;
    app.is_sending_email = true;

    let Some(account) = app.current_account.clone() else {
        app.set_error("No account selected");
        return;
    };

    let draft = app.compose_draft.clone();
    let draft_path = app.compose_draft_path.clone();

    *task = Some(tokio::spawn(async move {
        services::send_draft(&account, draft).await?;
        Ok(draft_path)
    }));
}

pub(in crate::tui::runtime) async fn handle_send_result(
    app: &mut App,
    task: &mut Option<ComposeSendTask>,
) -> Result<()> {
    let Some(send_task) = task.as_mut() else {
        return Ok(());
    };

    if !send_task.is_finished() {
        return Ok(());
    }

    let Some(send_task) = task.take() else {
        return Ok(());
    };

    app.is_sending_email = false;

    match send_task.await {
        Ok(Ok(draft_path)) => {
            if let Some(path) = draft_path {
                storage::delete_draft_file(&path).ok();
            }
            app.set_status("Email sent successfully!");
            let next_screen = app.pre_compose_screen.take().unwrap_or(Screen::Inbox);
            app.navigate_to(next_screen);
        }
        Ok(Err(error)) => {
            let msg = format_send_error(app.current_account.as_ref(), &error);
            app.set_error(msg);
        }
        Err(error) => {
            app.set_error(format!("Task error: {}", error));
        }
    }

    Ok(())
}
