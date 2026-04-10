use ratatui::style::Modifier;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(crate) fn truncate_with_ellipsis(value: &str, max: usize) -> String {
    if UnicodeWidthStr::width(value) <= max {
        return value.to_string();
    }

    if max <= 1 {
        return "…".to_string();
    }

    let target = max - 1;
    let mut result = String::new();
    let mut used = 0;

    for ch in value.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > target {
            break;
        }
        result.push(ch);
        used += w;
    }

    format!("{}…", result)
}

pub(crate) fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

pub(crate) fn display_subject(subject: &str) -> (&str, bool) {
    if subject.trim().is_empty() {
        ("No Subject", true)
    } else {
        (subject, false)
    }
}

pub(crate) fn subject_modifier(base: Modifier, is_seen: bool, has_empty_subject: bool) -> Modifier {
    if is_seen || has_empty_subject {
        base | Modifier::DIM
    } else {
        base
    }
}

pub(crate) fn sanitize_email_body(body: &str) -> String {
    let mut sanitized = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\r' if matches!(chars.peek(), Some('\n')) => {}
            '\r' => sanitized.push('\n'),
            '\t' => sanitized.push_str("    "),
            '\n' => sanitized.push('\n'),
            _ if ch.is_control() => {}
            _ => sanitized.push(ch),
        }
    }

    sanitized
}
