use std::char;

use html2text::from_read;

use super::text::normalize_terminal_text;

const HTML_RENDER_WIDTH: usize = 120;

pub(super) fn render_html_to_terminal_text(html: &str) -> String {
    let rendered =
        from_read(html.as_bytes(), HTML_RENDER_WIDTH).unwrap_or_else(|_| strip_html_fallback(html));

    normalize_terminal_text(rendered.as_str(), false)
}

fn strip_html_fallback(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }

    normalize_terminal_text(decode_html_entities(text.as_str()).as_str(), true)
}

fn decode_html_entities(text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '&' {
            decoded.push(ch);
            continue;
        }

        let mut entity = String::new();
        let mut terminated = false;

        while let Some(&next) = chars.peek() {
            if next == ';' {
                entity.push(next);
                chars.next();
                terminated = true;
                break;
            }

            if next.is_whitespace() || next == '<' || entity.len() >= 16 {
                break;
            }

            entity.push(next);
            chars.next();
        }

        if terminated {
            if let Some(replacement) = decode_html_entity(entity.as_str()) {
                decoded.push_str(replacement.as_str());
                continue;
            }
        }

        decoded.push('&');
        decoded.push_str(entity.as_str());
    }

    decoded
}

fn decode_html_entity(entity: &str) -> Option<String> {
    let name = entity.strip_suffix(';')?;

    if let Some(hex) = name.strip_prefix("#x").or_else(|| name.strip_prefix("#X")) {
        return u32::from_str_radix(hex, 16)
            .ok()
            .and_then(char::from_u32)
            .map(|value| value.to_string());
    }

    if let Some(decimal) = name.strip_prefix('#') {
        return decimal
            .parse::<u32>()
            .ok()
            .and_then(char::from_u32)
            .map(|value| value.to_string());
    }

    match name {
        "nbsp" => Some(" ".to_string()),
        "lt" => Some("<".to_string()),
        "gt" => Some(">".to_string()),
        "amp" => Some("&".to_string()),
        "quot" => Some("\"".to_string()),
        "apos" | "#39" => Some("'".to_string()),
        "bull" | "middot" => Some("*".to_string()),
        "ndash" | "mdash" => Some("-".to_string()),
        "hellip" => Some("...".to_string()),
        _ => None,
    }
}
