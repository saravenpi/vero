mod types;

use anyhow::{Context, Result};

#[cfg(test)]
pub use types::AutoRefresh;
pub use types::{Account, ImapConfig, SmtpConfig, VeroConfig};

impl VeroConfig {
    pub fn load() -> Result<Self> {
        let home = dirs::home_dir().context("Unable to find home directory")?;
        let config_path = home.join(".vero.yml");

        let contents = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Unable to read config file at {:?}", config_path))?;

        let mut config: VeroConfig =
            serde_yaml::from_str(&contents).context("Unable to parse config file")?;

        if config.accounts.is_empty() {
            anyhow::bail!("No accounts configured in {:?}", config_path);
        }

        for account in &mut config.accounts {
            if account.email.is_empty() {
                anyhow::bail!("Account with empty email found");
            }
            if account.imap.host.is_empty() {
                anyhow::bail!("Account {} is missing IMAP host", account.email);
            }
            if account.smtp.host.is_empty() {
                anyhow::bail!("Account {} is missing SMTP host", account.email);
            }

            if account.imap.user.is_none() {
                account.imap.user = Some(account.email.clone());
            }
            if account.smtp.user.is_none() {
                account.smtp.user = Some(account.email.clone());
            }
        }

        if config.download_folder.is_none() {
            config.download_folder = Some(home.join("Downloads").to_string_lossy().to_string());
        } else if let Some(ref folder) = config.download_folder {
            config.download_folder = Some(expand_path(folder, &home));
        }

        let valid_views = ["unseen", "seen", "all"];
        if !valid_views.contains(&config.inbox_view.as_str()) {
            anyhow::bail!(
                "Invalid inbox_view '{}', must be 'unseen', 'seen', or 'all'",
                config.inbox_view
            );
        }

        Ok(config)
    }
}

fn expand_path(path: &str, home: &std::path::Path) -> String {
    let expanded = shellexpand::tilde(path);
    if expanded.starts_with('/') {
        expanded.to_string()
    } else {
        home.join(expanded.as_ref()).to_string_lossy().to_string()
    }
}
