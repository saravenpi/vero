use super::*;
use crate::{
    models::ViewMode,
    tui::test_support::{test_app, test_draft_named, test_email},
};

#[test]
fn double_g_moves_inbox_selection_to_top() {
    let mut app = test_app();
    app.screen = Screen::Inbox;
    app.inbox_emails = (1..=5).map(test_email).collect();
    app.inbox_selected = 4;
    app.inbox_list_offset = 4;

    assert!(app.handle_list_jump_key(KeyCode::Char('g')));
    assert_eq!(app.inbox_selected, 4);

    assert!(app.handle_list_jump_key(KeyCode::Char('g')));
    assert_eq!(app.inbox_selected, 0);
    assert_eq!(app.inbox_list_offset, 0);
}

#[test]
fn double_g_moves_sent_selection_to_top() {
    let mut app = test_app();
    app.screen = Screen::Sent;
    app.sent_emails = (1..=5).map(test_email).collect();
    app.sent_selected = 4;
    app.sent_list_offset = 4;

    assert!(app.handle_list_jump_key(KeyCode::Char('g')));
    assert!(app.handle_list_jump_key(KeyCode::Char('g')));
    assert_eq!(app.sent_selected, 0);
    assert_eq!(app.sent_list_offset, 0);
}

#[test]
fn double_g_moves_drafts_selection_to_top() {
    let mut app = test_app();
    app.screen = Screen::Drafts;
    app.drafts = vec![
        test_draft_named("one"),
        test_draft_named("two"),
        test_draft_named("three"),
    ];
    app.drafts_selected = 2;
    app.drafts_list_offset = 2;

    assert!(app.handle_list_jump_key(KeyCode::Char('g')));
    assert!(app.handle_list_jump_key(KeyCode::Char('g')));
    assert_eq!(app.drafts_selected, 0);
    assert_eq!(app.drafts_list_offset, 0);
}

#[test]
fn uppercase_g_moves_inbox_selection_to_bottom() {
    let mut app = test_app();
    app.screen = Screen::Inbox;
    app.inbox_emails = (1..=5).map(test_email).collect();
    app.inbox_selected = 1;

    assert!(app.handle_list_jump_key(KeyCode::Char('G')));
    assert_eq!(app.inbox_selected, 4);
    assert_eq!(app.inbox_list_offset, 4);
}

#[test]
fn uppercase_g_moves_sent_selection_to_bottom() {
    let mut app = test_app();
    app.screen = Screen::Sent;
    app.sent_emails = (1..=5).map(test_email).collect();
    app.sent_selected = 1;

    assert!(app.handle_list_jump_key(KeyCode::Char('G')));
    assert_eq!(app.sent_selected, 4);
    assert_eq!(app.sent_list_offset, 4);
}

#[test]
fn uppercase_g_moves_drafts_selection_to_bottom() {
    let mut app = test_app();
    app.screen = Screen::Drafts;
    app.drafts = vec![
        test_draft_named("one"),
        test_draft_named("two"),
        test_draft_named("three"),
    ];
    app.drafts_selected = 0;

    assert!(app.handle_list_jump_key(KeyCode::Char('G')));
    assert_eq!(app.drafts_selected, 2);
    assert_eq!(app.drafts_list_offset, 2);
}

#[test]
fn non_g_key_clears_pending_list_jump() {
    let mut app = test_app();
    app.screen = Screen::Inbox;
    app.inbox_emails = (1..=3).map(test_email).collect();
    app.inbox_selected = 2;

    assert!(app.handle_list_jump_key(KeyCode::Char('g')));
    assert!(!app.handle_list_jump_key(KeyCode::Enter));
    assert!(app.handle_list_jump_key(KeyCode::Char('g')));
    assert_eq!(app.inbox_selected, 2);
}

#[test]
fn double_g_moves_inbox_detail_scroll_to_top() {
    let mut app = test_app();
    app.screen = Screen::Inbox;
    app.inbox_view_mode = ViewMode::Detail;
    app.inbox_scroll_offset = 9;

    assert!(app.handle_list_jump_key(KeyCode::Char('g')));
    assert!(app.handle_list_jump_key(KeyCode::Char('g')));
    assert_eq!(app.inbox_scroll_offset, 0);
}

#[test]
fn uppercase_g_moves_sent_detail_scroll_to_bottom_marker() {
    let mut app = test_app();
    app.screen = Screen::Sent;
    app.sent_view_mode = ViewMode::Detail;
    app.sent_scroll_offset = 3;

    assert!(app.handle_list_jump_key(KeyCode::Char('G')));
    assert_eq!(app.sent_scroll_offset, usize::MAX);
}
