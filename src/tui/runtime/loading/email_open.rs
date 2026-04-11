use anyhow::Result;

use crate::models::ViewMode;
use crate::services;
use crate::tui::App;

pub(in crate::tui::runtime) type InboxOpenTask =
    tokio::task::JoinHandle<Result<crate::models::Email>>;

pub(in crate::tui::runtime) fn maybe_spawn_inbox_open(
    app: &mut App,
    task: &mut Option<InboxOpenTask>,
) {
    if !app.needs_inbox_open || task.is_some() {
        return;
    }

    app.needs_inbox_open = false;

    let Some(account) = app.current_account.clone() else {
        app.set_error("No account selected");
        app.inbox_open_loading = false;
        return;
    };

    let Some(email) = app.inbox_open_pending_email.take() else {
        app.inbox_open_loading = false;
        return;
    };

    *task = Some(tokio::spawn(async move {
        services::read_loaded_inbox_email(&account, email).await
    }));
}

pub(in crate::tui::runtime) async fn handle_inbox_open_result(
    app: &mut App,
    task: &mut Option<InboxOpenTask>,
) -> Result<()> {
    let Some(open_task) = task.as_mut() else {
        return Ok(());
    };

    if !open_task.is_finished() {
        return Ok(());
    }

    let Some(open_task) = task.take() else {
        return Ok(());
    };

    app.inbox_open_loading = false;

    match open_task.await {
        Ok(Ok(email)) => {
            app.update_inbox_email(email);
            app.inbox_scroll_offset = 0;
            app.inbox_view_mode = ViewMode::Detail;
            app.needs_full_redraw = true;
        }
        Ok(Err(error)) => {
            app.set_error(format!("Failed to open email: {}", error));
        }
        Err(error) => {
            app.set_error(format!("Task error: {}", error));
        }
    }

    Ok(())
}
