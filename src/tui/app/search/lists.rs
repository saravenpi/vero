use std::path::PathBuf;

use crate::models::{Email, EmailDraft};
use crate::tui::app::App;

use super::matchers::{
    draft_matches_query, email_matches_query, sent_email_has_key, sent_email_key,
    sent_email_matches_query, SentSelectionKey,
};

impl App {
    pub(crate) fn inbox_visible_indices(&self) -> Vec<usize> {
        self.inbox_emails
            .iter()
            .enumerate()
            .filter(|(_, email)| email_matches_query(self.inbox_search.display_query(), email))
            .map(|(index, _)| index)
            .collect()
    }

    pub(crate) fn inbox_visible_len(&self) -> usize {
        self.inbox_visible_indices().len()
    }

    pub(crate) fn selected_inbox_index(&self) -> Option<usize> {
        self.inbox_visible_indices()
            .get(self.inbox_selected)
            .copied()
    }

    pub(crate) fn selected_inbox_email(&self) -> Option<&Email> {
        let index = self.selected_inbox_index()?;
        self.inbox_emails.get(index)
    }

    pub(crate) fn selected_inbox_uid(&self) -> Option<u32> {
        self.selected_inbox_email().map(|email| email.uid)
    }

    pub(crate) fn restore_inbox_search_selection(&mut self, selected_uid: Option<u32>) {
        if let Some(uid) = selected_uid {
            let visible_indices = self.inbox_visible_indices();
            if let Some(index) = visible_indices
                .iter()
                .position(|candidate| self.inbox_emails[*candidate].uid == uid)
            {
                self.inbox_selected = index;
                self.inbox_list_offset = self.inbox_list_offset.min(self.inbox_selected);
                return;
            }
        }

        self.clamp_inbox_selection();
    }

    pub(crate) fn sent_visible_indices(&self) -> Vec<usize> {
        self.sent_emails
            .iter()
            .enumerate()
            .filter(|(_, email)| sent_email_matches_query(self.sent_search.display_query(), email))
            .map(|(index, _)| index)
            .collect()
    }

    pub(crate) fn sent_visible_len(&self) -> usize {
        self.sent_visible_indices().len()
    }

    pub(crate) fn selected_sent_index(&self) -> Option<usize> {
        self.sent_visible_indices().get(self.sent_selected).copied()
    }

    pub(crate) fn selected_sent_email(&self) -> Option<&Email> {
        let index = self.selected_sent_index()?;
        self.sent_emails.get(index)
    }

    pub(crate) fn replace_sent_emails(&mut self, emails: Vec<Email>) {
        let selected_key = self.selected_sent_key();
        self.sent_emails = emails;
        self.restore_sent_selection(selected_key);
    }

    pub(crate) fn drafts_visible_indices(&self) -> Vec<usize> {
        self.drafts
            .iter()
            .enumerate()
            .filter(|(_, (path, draft))| {
                draft_matches_query(self.drafts_search.display_query(), path, draft)
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub(crate) fn drafts_visible_len(&self) -> usize {
        self.drafts_visible_indices().len()
    }

    pub(crate) fn selected_draft_index(&self) -> Option<usize> {
        self.drafts_visible_indices()
            .get(self.drafts_selected)
            .copied()
    }

    pub(crate) fn selected_draft(&self) -> Option<&(PathBuf, EmailDraft)> {
        let index = self.selected_draft_index()?;
        self.drafts.get(index)
    }

    pub(crate) fn replace_drafts(&mut self, drafts: Vec<(PathBuf, EmailDraft)>) {
        let selected_path = self.selected_draft_path();
        self.drafts = drafts;
        self.restore_draft_selection(selected_path);
    }

    pub(super) fn restore_sent_selection(&mut self, selected_key: Option<SentSelectionKey>) {
        if let Some(key) = selected_key {
            let visible_indices = self.sent_visible_indices();
            if let Some(index) = visible_indices
                .iter()
                .position(|candidate| sent_email_has_key(&self.sent_emails[*candidate], &key))
            {
                self.sent_selected = index;
                self.sent_list_offset = self.sent_list_offset.min(self.sent_selected);
                return;
            }
        }

        self.clamp_sent_selection();
    }

    pub(super) fn restore_draft_selection(&mut self, selected_path: Option<PathBuf>) {
        if let Some(path) = selected_path {
            let visible_indices = self.drafts_visible_indices();
            if let Some(index) = visible_indices
                .iter()
                .position(|candidate| self.drafts[*candidate].0 == path)
            {
                self.drafts_selected = index;
                self.drafts_list_offset = self.drafts_list_offset.min(self.drafts_selected);
                return;
            }
        }

        self.clamp_drafts_selection();
    }

    pub(super) fn selected_sent_key(&self) -> Option<SentSelectionKey> {
        self.selected_sent_email().map(sent_email_key)
    }

    pub(super) fn selected_draft_path(&self) -> Option<PathBuf> {
        self.selected_draft()
            .map(|draft: &(PathBuf, EmailDraft)| draft.0.clone())
    }
}
