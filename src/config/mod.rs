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

    pub fn editor_command(&self) -> Option<String> {
        configured_command(self.editor.as_deref()).or_else(editor_from_env)
    }

    pub fn viewer_command(&self) -> Option<String> {
        configured_command(self.viewer.as_deref()).or_else(editor_from_env)
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

fn configured_command(command: Option<&str>) -> Option<String> {
    command
        .map(str::trim)
        .filter(|command| !command.is_empty())
        .map(ToOwned::to_owned)
}

fn editor_from_env() -> Option<String> {
    configured_command(std::env::var("EDITOR").ok().as_deref())
}

#[cfg(test)]
mod tests {
    use crate::config::{Account, AutoRefresh, ImapConfig, SmtpConfig, VeroConfig};
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn test_config() -> VeroConfig {
        VeroConfig {
            accounts: vec![Account {
                email: "test@example.com".to_string(),
                imap: ImapConfig {
                    user: Some("test@example.com".to_string()),
                    password: "secret".to_string(),
                    host: "imap.example.com".to_string(),
                    port: 993,
                },
                smtp: SmtpConfig {
                    user: Some("test@example.com".to_string()),
                    password: "secret".to_string(),
                    host: "smtp.example.com".to_string(),
                    port: 465,
                },
            }],
            download_folder: None,
            inbox_view: "all".to_string(),
            auto_refresh: AutoRefresh { seconds: 0 },
            viewer: None,
            editor: None,
        }
    }

    #[test]
    fn editor_command_prefers_config_value() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("EDITOR", "nano");

        let mut config = test_config();
        config.editor = Some("nvim".to_string());

        assert_eq!(config.editor_command().as_deref(), Some("nvim"));

        std::env::remove_var("EDITOR");
    }

    #[test]
    fn editor_command_falls_back_to_editor_env() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("EDITOR", "hx");

        let config = test_config();

        assert_eq!(config.editor_command().as_deref(), Some("hx"));

        std::env::remove_var("EDITOR");
    }

    #[test]
    fn viewer_command_uses_viewer_then_editor_env_only() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("EDITOR", "hx");

        let mut config = test_config();
        config.editor = Some("nvim".to_string());

        assert_eq!(config.viewer_command().as_deref(), Some("hx"));

        config.viewer = Some("less".to_string());
        assert_eq!(config.viewer_command().as_deref(), Some("less"));

        std::env::remove_var("EDITOR");
    }
}
