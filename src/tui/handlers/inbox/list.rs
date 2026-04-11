use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::models::InboxFilter;
use crate::services;
use crate::tui::app::Screen;
use crate::tui::App;

pub(super) async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => app.navigate_to(Screen::AccountSelection),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Enter => open_selected_email(app),
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

fn open_selected_email(app: &mut App) {
    if app.current_account.is_none() {
        app.set_error("No account selected");
        return;
    }

    let Some(email) = app.selected_inbox_email().cloned() else {
        return;
    };

    app.cancel_inbox_load = true;
    app.inbox_loading = false;
    app.inbox_open_loading = true;
    app.inbox_open_pending_email = Some(email);
    app.needs_inbox_open = true;
}

async fn delete_selected_email(app: &mut App) -> Result<()> {
    let Some(account) = app.current_account.clone() else {
        app.set_error("No account selected");
        return Ok(());
    };

    let Some(email) = app.selected_inbox_email().cloned() else {
        return Ok(());
    };

    services::delete_loaded_inbox_email(&account, &email).await?;

    app.remove_inbox_email(email.uid);
    app.set_status("Email deleted.");

    Ok(())
}

fn set_filter(app: &mut App, filter: InboxFilter) {
    app.set_inbox_filter(filter);
}

fn cycle_filter_next(app: &mut App) {
    let filter = match app.inbox_filter {
        InboxFilter::All => InboxFilter::Unseen,
        InboxFilter::Unseen => InboxFilter::Seen,
        InboxFilter::Seen => InboxFilter::All,
    };
    set_filter(app, filter);
}

fn cycle_filter_prev(app: &mut App) {
    let filter = match app.inbox_filter {
        InboxFilter::All => InboxFilter::Seen,
        InboxFilter::Unseen => InboxFilter::All,
        InboxFilter::Seen => InboxFilter::Unseen,
    };
    set_filter(app, filter);
}

fn refresh(app: &mut App) {
    app.needs_inbox_load = true;
    app.inbox_loading = true;
}
