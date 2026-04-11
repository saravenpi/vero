use crossterm::event::{KeyCode, KeyEvent};

use crate::models::ViewMode;
use crate::tui::app::{App, FocusedElement, Screen};

use super::matchers::is_search_input;
use super::ListSearch;

impl App {
    pub(crate) fn inbox_search(&self) -> &ListSearch {
        &self.inbox_search
    }

    pub(crate) fn sent_search(&self) -> &ListSearch {
        &self.sent_search
    }

    pub(crate) fn drafts_search(&self) -> &ListSearch {
        &self.drafts_search
    }

    pub(crate) fn is_list_search_editing(&self) -> bool {
        self.current_list_search()
            .is_some_and(ListSearch::is_editing)
    }

    pub(crate) fn begin_list_search(&mut self) -> bool {
        self.clear_pending_list_navigation();

        match self.screen {
            Screen::Inbox
                if self.focused == FocusedElement::Content
                    && self.inbox_view_mode == ViewMode::List =>
            {
                self.inbox_search.is_editing = true;
                true
            }
            Screen::Drafts if self.focused == FocusedElement::Content => {
                self.drafts_search.is_editing = true;
                true
            }
            Screen::Sent
                if self.focused == FocusedElement::Content
                    && self.sent_view_mode == ViewMode::List =>
            {
                self.sent_search.is_editing = true;
                true
            }
            _ => false,
        }
    }

    pub(crate) fn clear_current_list_search(&mut self) -> bool {
        match self.screen {
            Screen::Inbox if self.inbox_view_mode == ViewMode::List => {
                if !self.inbox_search.is_active() && !self.inbox_search.is_editing {
                    return false;
                }
                let selected_uid = self.selected_inbox_uid();
                clear_search(&mut self.inbox_search);
                self.restore_inbox_search_selection(selected_uid);
                true
            }
            Screen::Drafts => {
                if !self.drafts_search.is_active() && !self.drafts_search.is_editing {
                    return false;
                }
                let selected_path = self.selected_draft_path();
                clear_search(&mut self.drafts_search);
                self.restore_draft_selection(selected_path);
                true
            }
            Screen::Sent if self.sent_view_mode == ViewMode::List => {
                if !self.sent_search.is_active() && !self.sent_search.is_editing {
                    return false;
                }
                let selected_key = self.selected_sent_key();
                clear_search(&mut self.sent_search);
                self.restore_sent_selection(selected_key);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn handle_list_search_key(&mut self, key: KeyEvent) -> bool {
        if !self.is_list_search_editing() {
            return false;
        }

        match self.screen {
            Screen::Inbox if self.inbox_view_mode == ViewMode::List => {
                let selected_uid = self.selected_inbox_uid();
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.inbox_search.is_editing = false;
                    }
                    KeyCode::Backspace => {
                        self.inbox_search.query.pop();
                        self.restore_inbox_search_selection(selected_uid);
                    }
                    KeyCode::Char(character) if is_search_input(key) => {
                        self.inbox_search.query.push(character);
                        self.restore_inbox_search_selection(selected_uid);
                    }
                    _ => {}
                }
                true
            }
            Screen::Drafts => {
                let selected_path = self.selected_draft_path();
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.drafts_search.is_editing = false;
                    }
                    KeyCode::Backspace => {
                        self.drafts_search.query.pop();
                        self.restore_draft_selection(selected_path);
                    }
                    KeyCode::Char(character) if is_search_input(key) => {
                        self.drafts_search.query.push(character);
                        self.restore_draft_selection(selected_path);
                    }
                    _ => {}
                }
                true
            }
            Screen::Sent if self.sent_view_mode == ViewMode::List => {
                let selected_key = self.selected_sent_key();
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.sent_search.is_editing = false;
                    }
                    KeyCode::Backspace => {
                        self.sent_search.query.pop();
                        self.restore_sent_selection(selected_key);
                    }
                    KeyCode::Char(character) if is_search_input(key) => {
                        self.sent_search.query.push(character);
                        self.restore_sent_selection(selected_key);
                    }
                    _ => {}
                }
                true
            }
            _ => false,
        }
    }

    pub(crate) fn current_list_search(&self) -> Option<&ListSearch> {
        match self.screen {
            Screen::Inbox if self.inbox_view_mode == ViewMode::List => Some(&self.inbox_search),
            Screen::Drafts => Some(&self.drafts_search),
            Screen::Sent if self.sent_view_mode == ViewMode::List => Some(&self.sent_search),
            _ => None,
        }
    }

    pub(crate) fn current_list_search_match_count(&self) -> Option<usize> {
        match self.screen {
            Screen::Inbox if self.inbox_view_mode == ViewMode::List => {
                Some(self.inbox_visible_len())
            }
            Screen::Drafts => Some(self.drafts_visible_len()),
            Screen::Sent if self.sent_view_mode == ViewMode::List => Some(self.sent_visible_len()),
            _ => None,
        }
    }
}

fn clear_search(search: &mut ListSearch) {
    search.query.clear();
    search.is_editing = false;
}

#[cfg(test)]
mod tests {
    use crate::tui::test_support::{test_app, test_email};

    use super::*;

    #[test]
    fn clearing_inbox_search_restores_selected_email() {
        let mut app = test_app();
        let mut alpha = test_email(1);
        alpha.subject = "Alpha".to_string();
        let mut beta = test_email(2);
        beta.subject = "Beta".to_string();
        app.inbox_emails = vec![alpha, beta];
        app.screen = Screen::Inbox;
        app.inbox_search.query = "beta".to_string();
        app.inbox_selected = 0;

        assert_eq!(app.selected_inbox_uid(), Some(2));

        app.clear_current_list_search();

        assert!(app.inbox_search.query.is_empty());
        assert_eq!(app.selected_inbox_uid(), Some(2));
    }
}
