const HEADER_PREFIXES: &[&str] = &[
    "from:", "to:", "cc:", "subject:", "date:", "sent:",
    "de:", "à:", "pour:", "objet:", "sujet:", "envoyé:",
    "von:", "an:", "betreff:", "gesendet:",
];

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    Normal,
    Blank,
    Attribution,
    Header,
    Quoted,
    Separator,
}

pub(super) fn classify_line(line: &str, lines: &[&str], index: usize) -> Kind {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return Kind::Blank;
    }

    if trimmed.starts_with("> ") || trimmed == ">" {
        return Kind::Quoted;
    }

    if trimmed.len() >= 7 && trimmed.chars().take(7).all(|c| c == '-' || c == '_') {
        return Kind::Separator;
    }

    if trimmed == "--" || trimmed == "-- " {
        return Kind::Separator;
    }

    let lower = trimmed.to_lowercase();

    if (lower.starts_with("le ") && lower.contains(" a écrit"))
        || (lower.starts_with("on ") && is_on_wrote(lower.as_str(), lines, index))
    {
        return Kind::Attribution;
    }

    if is_header(trimmed) && header_cluster_size(lines, index) >= 2 {
        return Kind::Header;
    }

    Kind::Normal
}

fn is_on_wrote(lower: &str, lines: &[&str], index: usize) -> bool {
    if lower.ends_with("wrote:") || lower.contains(" wrote:") {
        return true;
    }
    lines
        .get(index + 1)
        .map(|next| {
            let nl = next.trim().to_lowercase();
            nl.ends_with("wrote:") || nl.starts_with("wrote:")
        })
        .unwrap_or(false)
}

pub(super) fn is_header(trimmed: &str) -> bool {
    let lower = trimmed.to_lowercase().replace(" :", ":");
    HEADER_PREFIXES.iter().any(|p| lower.starts_with(p))
}

fn header_cluster_size(lines: &[&str], index: usize) -> usize {
    lines[index..]
        .iter()
        .take(5)
        .take_while(|l| !l.trim().is_empty())
        .filter(|l| is_header(l.trim()))
        .count()
}

pub(super) fn strip_quoted_content(body: &str) -> (&str, bool) {
    let lines: Vec<&str> = body.split('\n').collect();
    let mut last_non_blank_byte_end = 0;
    let mut byte_offset = 0;

    for (i, raw_line) in lines.iter().enumerate() {
        let line = raw_line.trim_end_matches('\r');
        let line_byte_end = byte_offset + raw_line.len();

        if is_quote_boundary(line, &lines, i) {
            if last_non_blank_byte_end == 0 {
                return (body, false);
            }
            return (&body[..last_non_blank_byte_end], true);
        }

        if !line.trim().is_empty() {
            last_non_blank_byte_end = line_byte_end;
        }

        byte_offset += raw_line.len() + 1;
    }

    (body, false)
}

fn is_quote_boundary(line: &str, lines: &[&str], index: usize) -> bool {
    matches!(
        classify_line(line, lines, index),
        Kind::Quoted | Kind::Attribution | Kind::Separator
    ) || (is_header(line.trim()) && header_cluster_size(lines, index) >= 2)
}
