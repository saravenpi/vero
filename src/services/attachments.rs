use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::config::Account;

fn expand_folder(folder: &str) -> String {
    shellexpand::tilde(folder).into_owned()
}

pub async fn download_inbox_attachments(
    account: &Account,
    uid: u32,
    index: Option<usize>,
    folder: &str,
) -> Result<Vec<PathBuf>> {
    let all = crate::email::fetch_attachment_bytes(&account.imap, uid).await?;

    let selected: Vec<_> = match index {
        Some(i) => {
            let pair = all
                .into_iter()
                .nth(i)
                .context("Attachment index out of range")?;
            vec![pair]
        }
        None => all,
    };

    let folder = expand_folder(folder);
    let mut saved = Vec::new();
    for (attachment, bytes) in selected {
        let path = save_bytes(&attachment.filename, &bytes, &folder)?;
        saved.push(path);
    }

    Ok(saved)
}

fn save_bytes(filename: &str, bytes: &[u8], folder: &str) -> Result<PathBuf> {
    let folder = Path::new(folder);
    std::fs::create_dir_all(folder).context("Failed to create download directory")?;

    let base = if filename.is_empty() {
        "attachment"
    } else {
        filename
    };

    let mut path = folder.join(base);

    if path.exists() {
        let stem = Path::new(base)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("attachment");
        let ext = Path::new(base).extension().and_then(|e| e.to_str());

        let mut counter = 1u32;
        loop {
            let name = match ext {
                Some(e) => format!("{} ({}).{}", stem, counter, e),
                None => format!("{} ({})", stem, counter),
            };
            path = folder.join(&name);
            if !path.exists() {
                break;
            }
            counter += 1;
        }
    }

    std::fs::write(&path, bytes).context("Failed to write attachment")?;
    Ok(path)
}
