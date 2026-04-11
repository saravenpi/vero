use std::path::Path;

use chrono::{DateTime, Utc};
use crossterm::event::{KeyEvent, KeyModifiers};

use crate::models::{Email, EmailDraft};

#[derive(Clone)]
pub(super) struct SentSelectionKey {
    pub(super) timestamp: DateTime<Utc>,
    pub(super) subject: String,
    pub(super) from: String,
    pub(super) to: Option<String>,
}

pub(super) fn is_search_input(key: KeyEvent) -> bool {
    !key.modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
}

pub(super) fn email_matches_query(query: &str, email: &Email) -> bool {
    tokens_match_fields(
        &search_tokens(query),
        [email.subject.as_str(), email.from.as_str()],
    )
}

pub(super) fn sent_email_matches_query(query: &str, email: &Email) -> bool {
    tokens_match_fields(
        &search_tokens(query),
        [
            email.subject.as_str(),
            email.from.as_str(),
            email.to.as_deref().unwrap_or(""),
        ],
    )
}

pub(super) fn draft_matches_query(query: &str, path: &Path, draft: &EmailDraft) -> bool {
    let file_stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("");
    tokens_match_fields(
        &search_tokens(query),
        [draft.subject.as_str(), draft.to.as_str(), file_stem],
    )
}

pub(super) fn sent_email_key(email: &Email) -> SentSelectionKey {
    SentSelectionKey {
        timestamp: email.timestamp,
        subject: email.subject.clone(),
        from: email.from.clone(),
        to: email.to.clone(),
    }
}

pub(super) fn sent_email_has_key(email: &Email, key: &SentSelectionKey) -> bool {
    email.timestamp == key.timestamp
        && email.subject == key.subject
        && email.from == key.from
        && email.to == key.to
}

fn search_tokens(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(normalize_search_text)
        .filter(|token| !token.is_empty())
        .collect()
}

fn normalize_search_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| character.to_lowercase())
        .collect::<String>()
}

fn tokens_match_fields<'a>(tokens: &[String], fields: impl IntoIterator<Item = &'a str>) -> bool {
    if tokens.is_empty() {
        return true;
    }

    let fields = fields
        .into_iter()
        .map(normalize_search_text)
        .collect::<Vec<_>>();

    tokens
        .iter()
        .all(|token| fields.iter().any(|field| field.contains(token)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_support::{test_draft_named, test_email};

    #[test]
    fn email_search_matches_subject_and_sender_tokens_case_insensitively() {
        let mut email = test_email(1);
        email.subject = "Quarterly Invoice".to_string();
        email.from = "Alice Example <alice@example.com>".to_string();

        assert!(email_matches_query("invoice alice", &email));
    }

    #[test]
    fn sent_search_matches_recipient_tokens() {
        let mut email = test_email(1);
        email.to = Some("bob@example.com".to_string());

        assert!(sent_email_matches_query("bob", &email));
    }

    #[test]
    fn draft_search_matches_file_stem_when_subject_is_empty() {
        let (path, draft) = test_draft_named("follow-up");

        assert!(draft_matches_query("follow", &path, &draft));
    }
}
