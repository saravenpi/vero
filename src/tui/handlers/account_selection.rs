use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::models::InboxFilter;
use crate::tui::app::Screen;
use crate::tui::App;

pub async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Enter => {
            if app.account_selected < app.config.accounts.len() {
                app.current_account = Some(app.config.accounts[app.account_selected].clone());
                app.inbox_filter = InboxFilter::from_str(&app.config.inbox_view);
                app.inbox_cached_emails.clear();
                app.inbox_cache_loaded = false;
                app.inbox_emails.clear();
                app.inbox_unseen_count = 0;
                app.navigate_to(Screen::Inbox);
            }
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::handle;
    use crate::config::{Account, AutoRefresh, ImapConfig, SmtpConfig, VeroConfig};
    use crate::models::InboxFilter;
    use crate::tui::{app::Screen, App};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn test_app() -> App {
        App::new(VeroConfig {
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
            inbox_view: "seen".to_string(),
            auto_refresh: AutoRefresh { seconds: 0 },
            viewer: None,
            editor: None,
        })
    }

    #[tokio::test]
    async fn selecting_account_uses_configured_inbox_filter() {
        let mut app = test_app();
        app.screen = Screen::AccountSelection;
        app.current_account = None;
        app.inbox_filter = InboxFilter::All;

        handle(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            .await
            .unwrap();

        assert_eq!(app.screen, Screen::Inbox);
        assert_eq!(app.inbox_filter, InboxFilter::Seen);
    }
}
