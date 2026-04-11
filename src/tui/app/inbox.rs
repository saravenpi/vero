use crate::models::{Email, InboxFilter};
use crate::services::InboxSnapshot;

use super::App;

impl App {
    pub fn apply_inbox_snapshot(&mut self, mut snapshot: InboxSnapshot) {
        let selected_uid = self.selected_inbox_uid();

        merge_loaded_email_bodies(&self.inbox_cached_emails, &mut snapshot.emails);
        self.inbox_cached_emails = snapshot.emails;
        self.inbox_cache_loaded = true;
        self.inbox_unseen_count = snapshot.unseen_count;
        self.refresh_inbox_emails(selected_uid);
    }

    pub fn refresh_inbox_emails(&mut self, selected_uid: Option<u32>) {
        self.inbox_emails = self
            .inbox_cached_emails
            .iter()
            .filter(|email| self.inbox_filter.matches(email))
            .cloned()
            .collect();

        self.restore_inbox_search_selection(selected_uid);
    }

    pub fn set_inbox_filter(&mut self, filter: InboxFilter) {
        if self.inbox_filter == filter {
            return;
        }

        self.inbox_filter = filter;
        self.inbox_selected = 0;
        self.inbox_list_offset = 0;

        if self.inbox_cache_loaded {
            self.refresh_inbox_emails(None);
            self.needs_inbox_cache_load = false;
        } else {
            self.needs_inbox_cache_load = true;
        }

        self.needs_inbox_load = true;
        self.inbox_loading = true;
    }

    pub fn update_inbox_email(&mut self, email: Email) {
        if let Some(index) = self
            .inbox_cached_emails
            .iter()
            .position(|existing| existing.uid == email.uid)
        {
            let was_unseen = !self.inbox_cached_emails[index].is_seen;
            self.inbox_cached_emails[index] = email.clone();
            if was_unseen && self.inbox_cached_emails[index].is_seen {
                self.inbox_unseen_count = self.inbox_unseen_count.saturating_sub(1);
            }
        }

        if let Some(index) = self
            .inbox_emails
            .iter()
            .position(|existing| existing.uid == email.uid)
        {
            self.inbox_emails[index] = email;
        }
    }

    pub fn remove_inbox_email(&mut self, uid: u32) {
        let selected_uid = self
            .selected_inbox_uid()
            .filter(|selected_uid| *selected_uid != uid);

        if let Some(email) = self
            .inbox_cached_emails
            .iter()
            .find(|existing| existing.uid == uid)
        {
            if !email.is_seen {
                self.inbox_unseen_count = self.inbox_unseen_count.saturating_sub(1);
            }
        }

        self.inbox_cached_emails.retain(|email| email.uid != uid);
        self.refresh_inbox_emails(selected_uid);
    }
}

fn merge_loaded_email_bodies(existing_emails: &[Email], loaded_emails: &mut [Email]) {
    for email in loaded_emails {
        if let Some(existing) = existing_emails
            .iter()
            .find(|candidate| candidate.uid == email.uid)
        {
            if !existing.body.is_empty() {
                email.body = existing.body.clone();
                email.attachments = existing.attachments.clone();
            }
            email.is_seen |= existing.is_seen;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_support::{test_app, test_email};

    fn snapshot(emails: Vec<Email>) -> InboxSnapshot {
        let unseen_count = emails.iter().filter(|email| !email.is_seen).count();
        InboxSnapshot {
            emails,
            unseen_count,
        }
    }

    #[test]
    fn apply_inbox_snapshot_filters_from_full_cache() {
        let mut app = test_app();
        let mut seen_email = test_email(2);
        seen_email.is_seen = true;
        app.inbox_filter = InboxFilter::Seen;

        app.apply_inbox_snapshot(snapshot(vec![
            test_email(1),
            seen_email.clone(),
            test_email(3),
        ]));

        assert!(app.inbox_cache_loaded);
        assert_eq!(app.inbox_cached_emails.len(), 3);
        assert_eq!(app.inbox_unseen_count, 2);
        assert_eq!(app.inbox_emails.len(), 1);
        assert_eq!(app.inbox_emails[0].uid, seen_email.uid);
    }

    #[test]
    fn set_inbox_filter_uses_cached_emails_immediately() {
        let mut app = test_app();
        let mut seen_email = test_email(2);
        seen_email.is_seen = true;
        app.apply_inbox_snapshot(snapshot(vec![
            test_email(1),
            seen_email.clone(),
            test_email(3),
        ]));
        app.inbox_loading = false;
        app.needs_inbox_load = false;

        app.set_inbox_filter(InboxFilter::Seen);

        assert_eq!(app.inbox_filter, InboxFilter::Seen);
        assert_eq!(app.inbox_emails.len(), 1);
        assert_eq!(app.inbox_emails[0].uid, seen_email.uid);
        assert!(app.inbox_loading);
        assert!(app.needs_inbox_load);
        assert!(!app.needs_inbox_cache_load);
    }
}
