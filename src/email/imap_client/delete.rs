use anyhow::Result;
use futures::{pin_mut, StreamExt};

use crate::config::ImapConfig;

use super::session::login;

pub async fn delete_email(cfg: &ImapConfig, uid: u32) -> Result<()> {
    let mut session = login(cfg).await?;

    session.select("INBOX").await?;

    let mut store_stream = session
        .uid_store(uid.to_string(), "+FLAGS.SILENT (\\Deleted)")
        .await?;

    while let Some(update) = store_stream.next().await {
        update?;
    }
    drop(store_stream);

    let used_uid_expunge = match session.uid_expunge(uid.to_string()).await {
        Ok(expunge_stream) => {
            pin_mut!(expunge_stream);
            while let Some(expunged) = expunge_stream.next().await {
                expunged?;
            }
            true
        }
        Err(_) => false,
    };

    if !used_uid_expunge {
        let expunge_stream = session.expunge().await?;
        pin_mut!(expunge_stream);
        while let Some(expunged) = expunge_stream.next().await {
            expunged?;
        }
    }

    session.logout().await?;

    Ok(())
}
