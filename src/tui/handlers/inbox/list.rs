use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::models::{InboxFilter, ViewMode};
use crate::services;
use crate::tui::app::Screen;
use crate::tui::App;

pub(super) async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => app.navigate_to(Screen::Inbox),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Enter => open_selected_email(app).await?,
        KeyCode::Char('d') => delete_selected_email(app).await?,
        KeyCode::Char('u') => set_filter(app, InboxFilter::Unseen),
        KeyCode::Char('s') => set_filter(app, InboxFilter::Seen),
        KeyCode::Char('a') => set_filter(app, InboxFilter::All),
        KeyCode::Char('h') | KeyCode::Left => cycle_filter_prev(app),
        KeyCode::Char('l') | KeyCode::Right => cycle_filter_next(app),
        KeyCode::Char('r') => refresh(app),
        KeyCode::Tab => app.tab_next_screen(),
        KeyCode::BackTab => app.tab_prev_screen(),
        _ => {}
    }

    Ok(())
}

async fn open_selected_email(app: &mut App) -> Result<()> {
    if app.inbox_selected >= app.inbox_emails.len() {
        return Ok(());
    }

    let Some(account) = app.current_account.clone() else {
        app.set_error("No account selected");
        return Ok(());
    };

    let email = app.inbox_emails[app.inbox_selected].clone();
    let was_unseen = !email.is_seen;
    let email = services::read_loaded_inbox_email(&account, email).await?;

    app.inbox_emails[app.inbox_selected] = email;
    if was_unseen {
        app.inbox_unseen_count = app.inbox_unseen_count.saturating_sub(1);
    }
    app.inbox_scroll_offset = 0;
    app.inbox_view_mode = ViewMode::Detail;
    app.needs_full_redraw = true;

    Ok(())
}

async fn delete_selected_email(app: &mut App) -> Result<()> {
    if app.inbox_selected >= app.inbox_emails.len() {
        return Ok(());
    }

    let Some(account) = app.current_account.clone() else {
        app.set_error("No account selected");
        return Ok(());
    };

    let email = app.inbox_emails[app.inbox_selected].clone();
    services::delete_loaded_inbox_email(&account, &email).await?;

    app.inbox_emails.remove(app.inbox_selected);
    app.clamp_inbox_selection();
    app.set_status("Email deleted.");

    Ok(())
}

fn set_filter(app: &mut App, filter: InboxFilter) {
    app.inbox_filter = filter;
    app.inbox_selected = 0;
    app.inbox_list_offset = 0;
    app.needs_inbox_cache_load = true;
    refresh(app);
}

fn cycle_filter_next(app: &mut App) {
    let filter = match app.inbox_filter {
        InboxFilter::Unseen => InboxFilter::Seen,
        InboxFilter::Seen => InboxFilter::All,
        InboxFilter::All => InboxFilter::Unseen,
    };
    set_filter(app, filter);
}

fn cycle_filter_prev(app: &mut App) {
    let filter = match app.inbox_filter {
        InboxFilter::Unseen => InboxFilter::All,
        InboxFilter::Seen => InboxFilter::Unseen,
        InboxFilter::All => InboxFilter::Seen,
    };
    set_filter(app, filter);
}

fn refresh(app: &mut App) {
    app.needs_inbox_load = true;
    app.inbox_loading = true;
    app.inbox_error = None;
}
