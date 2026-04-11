use super::{
    inbox, render,
    utils::{display_subject, sanitize_email_body, subject_modifier},
};
use crate::{
    models::InboxFilter,
    tui::{
        app::{ComposeStep, Screen},
        test_support::{test_app, test_draft_index, test_email},
    },
};
use ratatui::{backend::TestBackend, style::Modifier, Terminal};

fn buffer_lines(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer();
    let width = buffer.area().width as usize;

    buffer
        .content()
        .chunks(width)
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol())
                .collect::<Vec<_>>()
                .join("")
                .trim_end()
                .to_string()
        })
        .collect()
}

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
fn inbox_title_shows_count() {
    let mut app = test_app();
    app.inbox_filter = InboxFilter::All;
    app.inbox_emails = vec![test_email(1)];

    let title = inbox::title(&app);

    assert_eq!(title, " ▼ Inbox (1) ");
}

#[test]
fn inbox_title_ignores_refresh_and_error_state() {
    let mut app = test_app();
    app.inbox_filter = InboxFilter::Unseen;
    app.inbox_emails = vec![test_email(1), test_email(2)];
    app.inbox_loading = true;
    app.inbox_error = Some("nope".to_string());

    assert_eq!(inbox::title(&app), " ▼ Inbox (2) ");
}

#[test]
fn inbox_tabs_start_with_configured_default_filter() {
    let mut app = test_app();
    app.config.inbox_view = "seen".to_string();

    let filters = inbox::tab_filters(&app);

    assert_eq!(
        filters,
        [InboxFilter::Seen, InboxFilter::All, InboxFilter::Unseen]
    );
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
    app.screen = Screen::Drafts;
    app.drafts = (1..=20).map(test_draft_index).collect();
    app.drafts_selected = 15;

    terminal.draw(|frame| render(frame, &mut app)).unwrap();

    assert!(app.drafts_list_offset > 0);
    assert!(app.drafts_list_offset <= app.drafts_selected);
}

#[test]
fn compose_preview_uses_clean_title_and_preserves_body_lines() {
    let backend = TestBackend::new(90, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = test_app();
    app.screen = Screen::Compose;
    app.compose_step = ComposeStep::Preview;
    app.compose_draft.to = "person@example.com".to_string();
    app.compose_draft.subject = "Preview subject".to_string();
    app.compose_draft.body = "line one\nline two".to_string();

    terminal.draw(|frame| render(frame, &mut app)).unwrap();

    let lines = buffer_lines(&terminal);

    assert!(lines.iter().any(|line| line.contains("Preview")));
    assert!(!lines.iter().any(|line| line.contains("Compose - Preview")));
    assert!(!lines.iter().any(|line| line.contains("Edit again")));
    assert!(lines
        .iter()
        .any(|line| line.contains("Body:")));
    assert!(lines.iter().any(|line| line.contains("line one")));
    assert!(lines.iter().any(|line| line.contains("line two")));
}
