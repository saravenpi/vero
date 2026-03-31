use crate::config::{Account, VeroConfig};
use crate::models::{Email, EmailDraft, InboxFilter, ViewMode};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    AccountSelection,
    Inbox,
    Sent,
    Compose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedElement {
    MenuBar,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeStep {
    Editing,
    Preview,
    NoEditor,
}

pub struct App {
    pub config: VeroConfig,
    pub current_account: Option<Account>,
    pub screen: Screen,
    pub focused: FocusedElement,
    pub should_quit: bool,

    pub account_selected: usize,

    pub menu_selected: usize,

    pub inbox_filter: InboxFilter,
    pub inbox_emails: Vec<Email>,
    pub inbox_selected: usize,
    pub inbox_view_mode: ViewMode,
    pub inbox_loading: bool,
    pub inbox_error: Option<String>,
    pub inbox_unseen_count: usize,
    pub inbox_scroll_offset: usize,

    pub sent_emails: Vec<Email>,
    pub sent_selected: usize,
    pub sent_view_mode: ViewMode,
    pub sent_loading: bool,
    pub sent_error: Option<String>,
    pub sent_scroll_offset: usize,

    pub compose_step: ComposeStep,
    pub compose_draft: EmailDraft,
    pub compose_draft_path: Option<PathBuf>,

    pub status_message: Option<String>,
    pub error_message: Option<String>,

    pub auto_refresh_counter: u64,

    pub needs_inbox_load: bool,
    pub needs_sent_load: bool,
    pub needs_editor_open: bool,
    pub needs_full_redraw: bool,

    pub cancel_inbox_load: bool,
    pub cancel_sent_load: bool,

    pub spinner_state: usize,
}

impl App {
    pub fn new(config: VeroConfig) -> Self {
        let current_account = if config.accounts.len() == 1 {
            Some(config.accounts[0].clone())
        } else {
            None
        };

        let screen = if current_account.is_some() {
            Screen::Inbox
        } else {
            Screen::AccountSelection
        };

        let inbox_filter = InboxFilter::from_str(&config.inbox_view);
        let should_load_inbox = current_account.is_some();

        Self {
            config,
            current_account,
            screen,
            focused: FocusedElement::MenuBar,
            should_quit: false,

            account_selected: 0,

            menu_selected: 0,

            inbox_filter,
            inbox_emails: Vec::new(),
            inbox_selected: 0,
            inbox_view_mode: ViewMode::List,
            inbox_loading: should_load_inbox,
            inbox_error: None,
            inbox_unseen_count: 0,
            inbox_scroll_offset: 0,

            sent_emails: Vec::new(),
            sent_selected: 0,
            sent_view_mode: ViewMode::List,
            sent_loading: false,
            sent_error: None,
            sent_scroll_offset: 0,

            compose_step: ComposeStep::Editing,
            compose_draft: EmailDraft::default(),
            compose_draft_path: None,

            status_message: None,
            error_message: None,

            auto_refresh_counter: 0,

            needs_inbox_load: should_load_inbox,
            needs_sent_load: false,
            needs_editor_open: false,
            needs_full_redraw: false,

            cancel_inbox_load: false,
            cancel_sent_load: false,

            spinner_state: 0,
        }
    }

    pub fn menu_items() -> Vec<&'static str> {
        vec!["Inbox", "Sent", "Compose"]
    }

    pub fn navigate_to(&mut self, screen: Screen) {
        self.screen = screen;
        self.error_message = None;

        match screen {
            Screen::Inbox => {
                self.menu_selected = 0;
                self.focused = FocusedElement::Content;
                self.inbox_selected = 0;
                self.needs_inbox_load = true;
                self.inbox_loading = true;
                self.inbox_error = None;
            }
            Screen::Sent => {
                self.menu_selected = 1;
                self.focused = FocusedElement::Content;
                self.sent_selected = 0;
                self.needs_sent_load = true;
                self.sent_loading = true;
                self.sent_error = None;
            }
            Screen::Compose => {
                self.menu_selected = 2;
                self.focused = FocusedElement::Content;
                self.compose_step = ComposeStep::Editing;
                self.compose_draft = EmailDraft::default();
                self.compose_draft_path = None;
                self.needs_editor_open = true;
            }
            Screen::AccountSelection => {
                self.focused = FocusedElement::Content;
            }
        }
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
            2 => self.navigate_to(Screen::Compose),
            _ => {}
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focused = match self.focused {
            FocusedElement::MenuBar => FocusedElement::Content,
            _ => FocusedElement::MenuBar,
        };
    }

    pub fn select_next(&mut self) {
        match self.screen {
            Screen::Inbox if self.inbox_view_mode == ViewMode::List => {
                if !self.inbox_emails.is_empty()
                    && self.inbox_selected < self.inbox_emails.len() - 1
                {
                    self.inbox_selected += 1;
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

    pub fn tick_auto_refresh(&mut self) -> bool {
        if self.config.auto_refresh.seconds > 0 && self.screen == Screen::Inbox {
            self.auto_refresh_counter += 1;
            if self.auto_refresh_counter >= self.config.auto_refresh.seconds * 10 {
                self.auto_refresh_counter = 0;
                return true;
            }
        }
        false
    }

    pub fn tick_spinner(&mut self) {
        self.spinner_state = (self.spinner_state + 1) % 10;
    }

    pub fn spinner_char(&self) -> &'static str {
        const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        SPINNER[self.spinner_state]
    }
}
