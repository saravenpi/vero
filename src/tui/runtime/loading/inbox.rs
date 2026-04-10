use anyhow::Result;

use crate::models::Email;
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

    match services::load_cached_inbox(account, app.inbox_filter) {
        Ok(snapshot) => {
            apply_inbox_snapshot(app, snapshot);
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
        let filter = app.inbox_filter;
        *task = Some(tokio::spawn(async move {
            services::load_inbox(&account, filter).await
        }));
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
            apply_inbox_snapshot(app, snapshot);
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

fn apply_inbox_snapshot(app: &mut App, snapshot: InboxSnapshot) {
    let selected_uid = app
        .inbox_emails
        .get(app.inbox_selected)
        .map(|email| email.uid);

    let mut emails = snapshot.emails;
    merge_loaded_email_bodies(&app.inbox_emails, &mut emails);
    app.inbox_emails = emails;
    restore_inbox_selection(app, selected_uid);
    app.clamp_inbox_selection();
    app.inbox_unseen_count = snapshot.unseen_count;
}

fn restore_inbox_selection(app: &mut App, selected_uid: Option<u32>) {
    if let Some(uid) = selected_uid {
        if let Some(index) = app.inbox_emails.iter().position(|e| e.uid == uid) {
            app.inbox_selected = index;
            return;
        }
    }

    if app.inbox_emails.is_empty() {
        app.inbox_selected = 0;
        app.inbox_list_offset = 0;
    } else if app.inbox_selected >= app.inbox_emails.len() {
        app.inbox_selected = app.inbox_emails.len() - 1;
    }
}

fn merge_loaded_email_bodies(existing_emails: &[Email], loaded_emails: &mut [Email]) {
    for email in loaded_emails {
        if let Some(existing) = existing_emails.iter().find(|e| e.uid == email.uid) {
            if !existing.body.is_empty() {
                email.body = existing.body.clone();
                email.attachments = existing.attachments.clone();
            }
            email.is_seen |= existing.is_seen;
        }
    }
}
