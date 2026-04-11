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
    assert_eq!(app.inbox_error.as_deref(), Some("old"));
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

#[tokio::test]
async fn slash_search_filters_inbox_rows_by_subject_and_sender() {
    let mut app = test_app();
    app.focused = FocusedElement::Content;
    app.inbox_view_mode = ViewMode::List;

    let mut alpha = crate::tui::test_support::test_email(1);
    alpha.subject = "Invoice".to_string();
    alpha.from = "Alice Example <alice@example.com>".to_string();

    let mut beta = crate::tui::test_support::test_email(2);
    beta.subject = "Status".to_string();
    beta.from = "Bob Example <bob@example.com>".to_string();

    app.inbox_emails = vec![alpha, beta];

    handle(
        &mut app,
        KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
    )
    .await
    .unwrap();
    handle(
        &mut app,
        KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
    )
    .await
    .unwrap();

    assert!(app.is_list_search_editing());
    assert_eq!(app.inbox_visible_len(), 1);
    assert_eq!(app.selected_inbox_uid(), Some(2));
}

#[tokio::test]
async fn escape_clears_active_search_before_leaving_inbox() {
    let mut app = test_app();
    app.focused = FocusedElement::Content;
    app.inbox_view_mode = ViewMode::List;
    app.inbox_emails = vec![crate::tui::test_support::test_email(1)];

    handle(
        &mut app,
        KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
    )
    .await
    .unwrap();
    handle(
        &mut app,
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
    )
    .await
    .unwrap();
    handle(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
        .await
        .unwrap();
    handle(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();

    assert_eq!(app.screen, crate::tui::app::Screen::Inbox);
    assert!(!app.inbox_search().is_active());

    handle(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.screen, crate::tui::app::Screen::AccountSelection);
}
