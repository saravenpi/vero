use super::{App, Screen};

const STATUS_TTL: u8 = 30;

impl App {
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
        self.error_message = None;
        self.status_ttl = STATUS_TTL;
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
        self.status_message = None;
        self.status_ttl = STATUS_TTL;
    }

    pub fn set_inbox_error(&mut self, msg: impl Into<String>) {
        self.inbox_error = Some(msg.into());
        self.error_message = None;
    }

    pub fn set_sent_error(&mut self, msg: impl Into<String>) {
        self.sent_error = Some(msg.into());
        self.error_message = None;
    }

    pub fn set_drafts_error(&mut self, msg: impl Into<String>) {
        self.drafts_error = Some(msg.into());
        self.error_message = None;
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

        if self.status_ttl > 0 {
            self.status_ttl -= 1;
            if self.status_ttl == 0 {
                self.status_message = None;
                self.error_message = None;
            }
        }
    }

    pub fn spinner_char(&self) -> &'static str {
        const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        SPINNER[self.spinner_state]
    }
}
