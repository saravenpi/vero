use super::{
    inbox, render,
    utils::{display_subject, sanitize_email_body, subject_modifier},
};
use crate::{
    models::InboxFilter,
    tui::test_support::{test_app, test_draft_index, test_email},
};
use ratatui::{backend::TestBackend, style::Modifier, Terminal};

#[test]
fn sanitize_email_body_normalizes_crlf_and_tabs() {
    let body = "one\r\ntwo\rthree\tfour";

    assert_eq!(sanitize_email_body(body), "one\ntwo\nthree    four");
}

#[test]
fn sanitize_email_body_drops_other_control_chars() {
    let body = "he\x00llo\x1b!";

    assert_eq!(sanitize_email_body(body), "hello!");
}

#[test]
fn inbox_title_shows_refresh_state_without_hiding_cached_emails() {
    let mut app = test_app();
    app.inbox_filter = InboxFilter::All;
    app.inbox_emails = vec![test_email(1)];
    app.inbox_loading = true;

    let title = inbox::title(&app);

    assert!(title.contains("Inbox - All (1)"));
    assert!(title.contains("Refreshing"));
}

#[test]
fn inbox_title_shows_refresh_failure_when_cached_emails_exist() {
    let mut app = test_app();
    app.inbox_filter = InboxFilter::Unseen;
    app.inbox_emails = vec![test_email(1), test_email(2)];
    app.inbox_loading = false;
    app.inbox_error = Some("nope".to_string());

    assert_eq!(inbox::title(&app), " ▼ Inbox - Unseen (2)  Refresh failed ");
}

#[test]
fn display_subject_uses_muted_placeholder_for_blank_subjects() {
    assert_eq!(display_subject(""), ("No Subject", true));
    assert_eq!(display_subject("   "), ("No Subject", true));
    assert_eq!(display_subject("hello"), ("hello", false));
}

#[test]
fn subject_modifier_dims_seen_and_blank_subjects() {
    assert_eq!(
        subject_modifier(Modifier::empty(), true, false),
        Modifier::DIM
    );
    assert_eq!(
        subject_modifier(Modifier::BOLD, false, true),
        Modifier::BOLD | Modifier::DIM
    );
    assert_eq!(
        subject_modifier(Modifier::REVERSED, false, false),
        Modifier::REVERSED
    );
}

#[test]
fn inbox_render_updates_list_offset_to_keep_selection_visible() {
    let backend = TestBackend::new(80, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = test_app();
    app.inbox_emails = (1..=20).map(test_email).collect();
    app.inbox_selected = 15;

    terminal.draw(|frame| render(frame, &mut app)).unwrap();

    assert!(app.inbox_list_offset > 0);
    assert!(app.inbox_list_offset <= app.inbox_selected);
}

#[test]
fn drafts_render_updates_list_offset_to_keep_selection_visible() {
    let backend = TestBackend::new(80, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = test_app();
    app.screen = crate::tui::app::Screen::Drafts;
    app.drafts = (1..=20).map(test_draft_index).collect();
    app.drafts_selected = 15;

    terminal.draw(|frame| render(frame, &mut app)).unwrap();

    assert!(app.drafts_list_offset > 0);
    assert!(app.drafts_list_offset <= app.drafts_selected);
}
