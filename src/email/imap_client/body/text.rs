pub(super) fn extract_text_part(mail: &mailparse::ParsedMail) -> Option<String> {
    let text = mail.get_body().ok()?;
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

pub(super) fn join_body_parts(parts: &[String]) -> Option<String> {
    let merged = parts
        .iter()
        .filter(|part| !part.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n\n");

    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

pub(super) fn push_unique_part(parts: &mut Vec<String>, part: String) {
    if parts.last().is_some_and(|existing| existing == &part) {
        return;
    }

    parts.push(part);
}

pub(super) fn normalize_plain_text(text: &str) -> String {
    normalize_terminal_text(text, false)
}

pub(super) fn normalize_terminal_text(text: &str, collapse_inline_whitespace: bool) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = Vec::new();
    let mut previous_blank = false;

    for raw_line in normalized.split('\n') {
        let line = clean_terminal_line(raw_line, collapse_inline_whitespace);

        if line.trim().is_empty() {
            if !lines.is_empty() && !previous_blank {
                lines.push(String::new());
                previous_blank = true;
            }
            continue;
        }

        lines.push(line);
        previous_blank = false;
    }

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}

fn clean_terminal_line(line: &str, collapse_inline_whitespace: bool) -> String {
    let mut cleaned = String::with_capacity(line.len());
    let mut last_was_space = false;

    for ch in line.chars() {
        match ch {
            '\t' if collapse_inline_whitespace => {
                if !last_was_space {
                    cleaned.push(' ');
                    last_was_space = true;
                }
            }
            '\t' => {
                cleaned.push_str("    ");
                last_was_space = false;
            }
            '\u{00a0}' => {
                if !last_was_space {
                    cleaned.push(' ');
                    last_was_space = true;
                }
            }
            '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{feff}' => {}
            _ if ch.is_control() => {}
            _ if collapse_inline_whitespace && ch.is_whitespace() => {
                if !last_was_space {
                    cleaned.push(' ');
                    last_was_space = true;
                }
            }
            _ => {
                cleaned.push(ch);
                last_was_space = ch == ' ';
            }
        }
    }

    if collapse_inline_whitespace {
        cleaned.trim().to_string()
    } else {
        cleaned.trim_end().to_string()
    }
}
