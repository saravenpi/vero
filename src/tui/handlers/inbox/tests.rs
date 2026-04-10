use super::handle;
use crate::config::{Account, AutoRefresh, ImapConfig, SmtpConfig, VeroConfig};
use crate::models::{InboxFilter, ViewMode};
use crate::tui::app::FocusedElement;
use crate::tui::App;
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
        inbox_view: "all".to_string(),
        auto_refresh: AutoRefresh { seconds: 0 },
        viewer: None,
        editor: None,
    })
}

#[tokio::test]
async fn leaving_detail_view_triggers_background_refresh() {
    let mut app = test_app();
    app.focused = FocusedElement::Content;
    app.inbox_view_mode = ViewMode::Detail;
    app.inbox_filter = InboxFilter::All;
    app.needs_inbox_load = false;
    app.inbox_loading = false;
    app.inbox_error = Some("old".to_string());
    app.inbox_scroll_offset = 7;

    handle(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();

    assert_eq!(app.inbox_view_mode, ViewMode::List);
    assert_eq!(app.inbox_scroll_offset, 0);
    assert!(app.needs_inbox_load);
    assert!(app.inbox_loading);
    assert!(app.inbox_error.is_none());
    assert!(app.needs_full_redraw);
}

#[tokio::test]
async fn detail_view_supports_gg_and_g_scroll_jumps() {
    let mut app = test_app();
    app.focused = FocusedElement::Content;
    app.inbox_view_mode = ViewMode::Detail;
    app.inbox_scroll_offset = 7;

    handle(
        &mut app,
        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE),
    )
    .await
    .unwrap();
    handle(
        &mut app,
        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE),
    )
    .await
    .unwrap();
    assert_eq!(app.inbox_scroll_offset, 0);

    handle(
        &mut app,
        KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT),
    )
    .await
    .unwrap();
    assert_eq!(app.inbox_scroll_offset, usize::MAX);
}
