use super::{App, Screen};
use crate::models::ViewMode;

impl App {
    pub fn select_next(&mut self) {
        match self.screen {
            Screen::Inbox if self.inbox_view_mode == ViewMode::List => {
                if !self.inbox_emails.is_empty()
                    && self.inbox_selected < self.inbox_emails.len() - 1
                {
                    self.inbox_selected += 1;
                }
            }
            Screen::Drafts => {
                if !self.drafts.is_empty() && self.drafts_selected < self.drafts.len() - 1 {
                    self.drafts_selected += 1;
                }
            }
            Screen::Sent if self.sent_view_mode == ViewMode::List => {
                if !self.sent_emails.is_empty() && self.sent_selected < self.sent_emails.len() - 1 {
                    self.sent_selected += 1;
                }
            }
            Screen::AccountSelection => {
                if self.account_selected < self.config.accounts.len() - 1 {
                    self.account_selected += 1;
                }
            }
            _ => {}
        }
    }

    pub fn select_previous(&mut self) {
        match self.screen {
            Screen::Inbox if self.inbox_view_mode == ViewMode::List => {
                if self.inbox_selected > 0 {
                    self.inbox_selected -= 1;
                }
            }
            Screen::Drafts => {
                if self.drafts_selected > 0 {
                    self.drafts_selected -= 1;
                }
            }
            Screen::Sent if self.sent_view_mode == ViewMode::List => {
                if self.sent_selected > 0 {
                    self.sent_selected -= 1;
                }
            }
            Screen::AccountSelection => {
                if self.account_selected > 0 {
                    self.account_selected -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn clamp_inbox_selection(&mut self) {
        clamp_position(
            &mut self.inbox_selected,
            &mut self.inbox_list_offset,
            self.inbox_emails.len(),
        );
    }

    pub fn clamp_drafts_selection(&mut self) {
        clamp_position(
            &mut self.drafts_selected,
            &mut self.drafts_list_offset,
            self.drafts.len(),
        );
    }

    pub fn clamp_sent_selection(&mut self) {
        clamp_position(
            &mut self.sent_selected,
            &mut self.sent_list_offset,
            self.sent_emails.len(),
        );
    }

    pub(crate) fn clear_pending_list_navigation(&mut self) {
        self.pending_list_navigation = None;
    }
}

fn clamp_position(selected: &mut usize, offset: &mut usize, len: usize) {
    if len == 0 {
        *selected = 0;
        *offset = 0;
        return;
    }

    *selected = (*selected).min(len - 1);
    *offset = (*offset).min(*selected);
}
