mod attachments;
mod body;
mod header;

use crate::tui::app::Screen;
use crate::tui::App;
use ratatui::{layout::Rect, Frame};

pub(crate) fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.screen {
        Screen::Inbox => {
            let email = app.selected_inbox_email().cloned();
            if app.inbox_show_attachments {
                attachments::render_attachment_list(
                    frame,
                    area,
                    email.as_ref(),
                    app.inbox_attachment_selected,
                );
            } else {
                body::render_email_detail(
                    frame,
                    area,
                    &mut app.inbox_scroll_offset,
                    email.as_ref(),
                    app.inbox_collapse_quotes,
                );
            }
        }
        Screen::Sent => {
            let email = app.selected_sent_email().cloned();
            body::render_email_detail(
                frame,
                area,
                &mut app.sent_scroll_offset,
                email.as_ref(),
                false,
            )
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::{
        models::ViewMode,
        tui::{
            app::Screen,
            test_support::{test_app, test_email},
        },
    };
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn short_email_body_clamps_scroll_offset() {
        let backend = TestBackend::new(60, 16);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = test_app();
        let mut email = test_email(1);
        email.body = "short body".to_string();
        app.screen = Screen::Inbox;
        app.inbox_view_mode = ViewMode::Detail;
        app.inbox_emails = vec![email];
        app.inbox_scroll_offset = 5;

        terminal
            .draw(|frame| render(frame, &mut app, frame.area()))
            .unwrap();

        assert_eq!(app.inbox_scroll_offset, 0);
    }

    #[test]
    fn detail_view_renders_scrollbar_symbols() {
        let backend = TestBackend::new(60, 16);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = test_app();
        let mut email = test_email(1);
        email.body = (1..=40)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        app.screen = Screen::Inbox;
        app.inbox_view_mode = ViewMode::Detail;
        app.inbox_emails = vec![email];
        app.inbox_scroll_offset = 8;

        terminal
            .draw(|frame| render(frame, &mut app, frame.area()))
            .unwrap();

        let buffer = terminal.backend().buffer();
        let has_thumb = buffer.content().iter().any(|cell| cell.symbol() == "█");
        let has_track = buffer.content().iter().any(|cell| cell.symbol() == "│");

        assert!(has_thumb);
        assert!(has_track);
    }
}
