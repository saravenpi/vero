use anyhow::Result;
use std::path::PathBuf;

use crate::services;
use crate::tui::App;

pub(in crate::tui::runtime) type AttachmentDownloadTask =
    tokio::task::JoinHandle<Result<Vec<PathBuf>>>;

pub(in crate::tui::runtime) fn maybe_spawn_attachment_download(
    app: &mut App,
    task: &mut Option<AttachmentDownloadTask>,
) {
    if !app.needs_attachment_download || task.is_some() {
        return;
    }

    app.needs_attachment_download = false;
    app.is_downloading_attachment = true;

    let Some(account) = app.current_account.clone() else {
        app.set_error("No account selected");
        app.is_downloading_attachment = false;
        return;
    };

    let Some(email) = app.selected_inbox_email() else {
        app.set_error("No email selected");
        app.is_downloading_attachment = false;
        return;
    };

    let uid = email.uid;
    let index = app.attachment_download_index;
    let folder = app
        .config
        .download_folder
        .clone()
        .unwrap_or_else(|| "~/Downloads".to_string());

    *task = Some(tokio::spawn(async move {
        services::download_inbox_attachments(&account, uid, index, &folder).await
    }));
}

pub(in crate::tui::runtime) async fn handle_attachment_download_result(
    app: &mut App,
    task: &mut Option<AttachmentDownloadTask>,
) -> Result<()> {
    let Some(download_task) = task.as_mut() else {
        return Ok(());
    };

    if !download_task.is_finished() {
        return Ok(());
    }

    let Some(download_task) = task.take() else {
        return Ok(());
    };

    app.is_downloading_attachment = false;

    match download_task.await {
        Ok(Ok(paths)) => {
            let msg = if paths.len() == 1 {
                let name = paths[0]
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");
                format!("Saved: {}", name)
            } else {
                format!("{} attachments saved", paths.len())
            };
            app.set_status(msg);
        }
        Ok(Err(error)) => {
            app.set_error(format!("Download failed: {}", error));
        }
        Err(error) => {
            app.set_error(format!("Task error: {}", error));
        }
    }

    Ok(())
}
