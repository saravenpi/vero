use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Deserialize)]
pub struct VeroConfig {
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub download_folder: Option<String>,
    #[serde(default = "default_inbox_view")]
    pub inbox_view: String,
    #[serde(default, deserialize_with = "deserialize_auto_refresh")]
    pub auto_refresh: AutoRefresh,
    #[serde(default)]
    pub viewer: Option<String>,
    #[serde(default)]
    pub editor: Option<String>,
}

fn default_inbox_view() -> String {
    "all".to_string()
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AutoRefresh {
    pub seconds: u64,
}

fn deserialize_auto_refresh<'de, D>(deserializer: D) -> Result<AutoRefresh, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AutoRefreshValue {
        Bool(bool),
        Int(i64),
    }

    match AutoRefreshValue::deserialize(deserializer)? {
        AutoRefreshValue::Bool(true) => Ok(AutoRefresh { seconds: 10 }),
        AutoRefreshValue::Bool(false) => Ok(AutoRefresh { seconds: 0 }),
        AutoRefreshValue::Int(n) => Ok(AutoRefresh {
            seconds: if n < 0 { 0 } else { n as u64 },
        }),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    pub email: String,
    pub imap: ImapConfig,
    pub smtp: SmtpConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImapConfig {
    #[serde(default)]
    pub user: Option<String>,
    pub password: String,
    pub host: String,
    #[serde(default = "default_imap_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    #[serde(default)]
    pub user: Option<String>,
    pub password: String,
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
}

fn default_imap_port() -> u16 {
    993
}

fn default_smtp_port() -> u16 {
    465
}
