use anyhow::{anyhow, Result};

use crate::config::{Account, VeroConfig};

#[derive(Debug, Clone, serde::Serialize)]
pub struct AccountSummary {
    pub index: usize,
    pub email: String,
}

pub fn list_accounts(config: &VeroConfig) -> Vec<AccountSummary> {
    config
        .accounts
        .iter()
        .enumerate()
        .map(|(index, account)| AccountSummary {
            index: index + 1,
            email: account.email.clone(),
        })
        .collect()
}

pub fn resolve_account(config: &VeroConfig, selector: Option<&str>) -> Result<Account> {
    if let Some(selector) = selector {
        if let Ok(index) = selector.parse::<usize>() {
            return config
                .accounts
                .get(index.saturating_sub(1))
                .cloned()
                .ok_or_else(|| anyhow!("Account index {} is out of range", index));
        }

        return config
            .accounts
            .iter()
            .find(|account| account.email == selector)
            .cloned()
            .ok_or_else(|| anyhow!("Account '{}' not found", selector));
    }

    if config.accounts.len() == 1 {
        return Ok(config.accounts[0].clone());
    }

    Err(anyhow!(
        "Multiple accounts configured. Use --account <email-or-index>."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AutoRefresh, ImapConfig, SmtpConfig};

    fn config() -> VeroConfig {
        VeroConfig {
            accounts: vec![
                Account {
                    email: "one@example.com".to_string(),
                    imap: ImapConfig {
                        user: Some("one@example.com".to_string()),
                        password: "pw".to_string(),
                        host: "imap.example.com".to_string(),
                        port: 993,
                    },
                    smtp: SmtpConfig {
                        user: Some("one@example.com".to_string()),
                        password: "pw".to_string(),
                        host: "smtp.example.com".to_string(),
                        port: 465,
                    },
                },
                Account {
                    email: "two@example.com".to_string(),
                    imap: ImapConfig {
                        user: Some("two@example.com".to_string()),
                        password: "pw".to_string(),
                        host: "imap.example.com".to_string(),
                        port: 993,
                    },
                    smtp: SmtpConfig {
                        user: Some("two@example.com".to_string()),
                        password: "pw".to_string(),
                        host: "smtp.example.com".to_string(),
                        port: 465,
                    },
                },
            ],
            download_folder: None,
            inbox_view: "all".to_string(),
            auto_refresh: AutoRefresh { seconds: 0 },
            viewer: None,
            editor: None,
        }
    }

    #[test]
    fn resolves_account_by_index() {
        let account = resolve_account(&config(), Some("2")).unwrap();
        assert_eq!(account.email, "two@example.com");
    }

    #[test]
    fn requires_selector_for_multiple_accounts() {
        let error = resolve_account(&config(), None).unwrap_err();
        assert!(error.to_string().contains("Multiple accounts configured"));
    }
}
