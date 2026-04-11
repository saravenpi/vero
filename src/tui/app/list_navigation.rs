use super::{App, ComposeStep, PendingListNavigation, Screen};
use crate::models::ViewMode;
use crossterm::event::KeyCode;

#[cfg(test)]
mod tests;

impl App {
    pub fn select_first(&mut self) {
        match self.screen {
            Screen::Inbox if self.inbox_view_mode == ViewMode::List => self.jump_to_inbox(0),
            Screen::Inbox if self.inbox_view_mode == ViewMode::Detail => {
                self.inbox_scroll_offset = 0
            }
            Screen::Drafts => self.jump_to_drafts(0),
            Screen::Sent if self.sent_view_mode == ViewMode::List => self.jump_to_sent(0),
            Screen::Sent if self.sent_view_mode == ViewMode::Detail => self.sent_scroll_offset = 0,
            Screen::Compose if self.compose_step == ComposeStep::Preview => {
                self.compose_preview_scroll_offset = 0;
            }
            Screen::AccountSelection => {
                self.account_selected = 0;
            }
            _ => {}
        }
    }

    pub fn select_last(&mut self) {
        match self.screen {
            Screen::Inbox if self.inbox_view_mode == ViewMode::List => {
                let last = self.inbox_visible_len().saturating_sub(1);
                self.jump_to_inbox(last);
            }
            Screen::Inbox if self.inbox_view_mode == ViewMode::Detail => {
                self.inbox_scroll_offset = usize::MAX;
            }
            Screen::Drafts => {
                let last = self.drafts_visible_len().saturating_sub(1);
                self.jump_to_drafts(last);
            }
            Screen::Sent if self.sent_view_mode == ViewMode::List => {
                let last = self.sent_visible_len().saturating_sub(1);
                self.jump_to_sent(last);
            }
            Screen::Sent if self.sent_view_mode == ViewMode::Detail => {
                self.sent_scroll_offset = usize::MAX;
            }
            Screen::Compose if self.compose_step == ComposeStep::Preview => {
                self.compose_preview_scroll_offset = usize::MAX;
            }
            Screen::AccountSelection => {
                self.account_selected = self.config.accounts.len().saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn handle_list_jump_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('G') => {
                self.clear_pending_list_navigation();
                self.select_last();
                true
            }
            KeyCode::Char('g') if self.pending_list_navigation.is_some() => {
                self.clear_pending_list_navigation();
                self.select_first();
                true
            }
            KeyCode::Char('g') => {
                self.pending_list_navigation = Some(PendingListNavigation::AwaitingSecondG);
                true
            }
            _ => {
                self.clear_pending_list_navigation();
                false
            }
        }
    }

    fn jump_to_inbox(&mut self, selected: usize) {
        let len = self.inbox_visible_len();
        if len == 0 {
            self.inbox_selected = 0;
            self.inbox_list_offset = 0;
            return;
        }

        self.inbox_selected = selected.min(len - 1);
        self.inbox_list_offset = self.inbox_selected;
    }

    fn jump_to_drafts(&mut self, selected: usize) {
        let len = self.drafts_visible_len();
        if len == 0 {
            self.drafts_selected = 0;
            self.drafts_list_offset = 0;
            return;
        }

        self.drafts_selected = selected.min(len - 1);
        self.drafts_list_offset = self.drafts_selected;
    }

    fn jump_to_sent(&mut self, selected: usize) {
        let len = self.sent_visible_len();
        if len == 0 {
            self.sent_selected = 0;
            self.sent_list_offset = 0;
            return;
        }

        self.sent_selected = selected.min(len - 1);
        self.sent_list_offset = self.sent_selected;
    }
}
