use super::{App, ComposeStep, FocusedElement, Screen};
use crate::models::{EmailDraft, ViewMode};
use std::path::PathBuf;

impl App {
    pub fn navigate_to(&mut self, screen: Screen) {
        let current_screen = self.screen;
        self.clear_pending_list_navigation();
        self.screen = screen;
        self.error_message = None;

        match screen {
            Screen::Inbox => {
                self.menu_selected = 0;
                self.focused = FocusedElement::Content;
                self.inbox_selected = 0;
                self.inbox_list_offset = 0;
                self.inbox_view_mode = ViewMode::List;
                self.inbox_scroll_offset = 0;
                self.needs_inbox_cache_load = true;
                self.needs_inbox_load = true;
                self.inbox_loading = true;
            }
            Screen::Drafts => {
                self.menu_selected = 2;
                self.focused = FocusedElement::Content;
                self.drafts_selected = 0;
                self.drafts_list_offset = 0;
                self.needs_drafts_load = true;
            }
            Screen::Sent => {
                self.menu_selected = 1;
                self.focused = FocusedElement::Content;
                self.sent_selected = 0;
                self.sent_list_offset = 0;
                self.sent_view_mode = ViewMode::List;
                self.sent_scroll_offset = 0;
                self.needs_sent_load = true;
                self.sent_loading = true;
            }
            Screen::Compose => {
                self.pre_compose_screen = Some(current_screen);
                self.menu_selected = 2;
                self.focused = FocusedElement::Content;
                self.compose_step = ComposeStep::Editing;
                self.compose_draft = EmailDraft::default();
                self.compose_draft_path = None;
                self.compose_preview_scroll_offset = 0;
                self.needs_editor_open = true;
            }
            Screen::Signatures => {
                self.menu_selected = 3;
                self.focused = FocusedElement::Content;
                self.needs_signature_load = true;
            }
            Screen::AccountSelection => {
                self.focused = FocusedElement::Content;
            }
        }
    }

    pub fn resume_draft(&mut self, path: PathBuf) {
        self.clear_pending_list_navigation();
        self.pre_compose_screen = Some(Screen::Drafts);
        self.screen = Screen::Compose;
        self.menu_selected = 2;
        self.focused = FocusedElement::Content;
        self.error_message = None;
        self.compose_step = ComposeStep::Editing;
        self.compose_draft = EmailDraft::default();
        self.compose_draft_path = Some(path);
        self.compose_preview_scroll_offset = 0;
        self.needs_editor_open = true;
    }

    pub fn menu_next(&mut self) {
        let items = Self::menu_items();
        if self.menu_selected < items.len() - 1 {
            self.menu_selected += 1;
        } else {
            self.menu_selected = 0;
        }
    }

    pub fn menu_previous(&mut self) {
        let items = Self::menu_items();
        if self.menu_selected > 0 {
            self.menu_selected -= 1;
        } else {
            self.menu_selected = items.len() - 1;
        }
    }

    pub fn menu_select(&mut self) {
        match self.menu_selected {
            0 => self.navigate_to(Screen::Inbox),
            1 => self.navigate_to(Screen::Sent),
            2 => self.navigate_to(Screen::Drafts),
            3 => self.navigate_to(Screen::Signatures),
            _ => {}
        }
    }

    pub fn tab_next_screen(&mut self) {
        match self.screen {
            Screen::Inbox => self.navigate_to(Screen::Sent),
            Screen::Sent => self.navigate_to(Screen::Drafts),
            Screen::Drafts => self.navigate_to(Screen::Signatures),
            Screen::Signatures => self.navigate_to(Screen::Inbox),
            Screen::Compose => self.navigate_to(Screen::Drafts),
            Screen::AccountSelection => {}
        }
    }

    pub fn tab_prev_screen(&mut self) {
        match self.screen {
            Screen::Inbox => self.navigate_to(Screen::Signatures),
            Screen::Sent => self.navigate_to(Screen::Inbox),
            Screen::Drafts => self.navigate_to(Screen::Sent),
            Screen::Signatures => self.navigate_to(Screen::Drafts),
            Screen::Compose => self.navigate_to(Screen::Drafts),
            Screen::AccountSelection => {}
        }
    }
}
