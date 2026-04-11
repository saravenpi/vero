use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

use crate::tui::ui::quote::{classify_line, is_header, Kind};
use crate::tui::ui::utils::{sanitize_line, sanitize_quoted_line};

pub(super) fn plain(body: &str) -> Text<'static> {
    body.split('\n')
        .map(|raw| Line::from(Span::raw(sanitize_line(raw))))
        .collect::<Vec<_>>()
        .into()
}

pub(super) fn styled(body: &str) -> Text<'static> {
    let raw_lines: Vec<&str> = body.split('\n').collect();
    let mut out: Vec<Line<'static>> = Vec::new();
    let mut prev_non_blank = Kind::Normal;

    for (i, raw) in raw_lines.iter().enumerate() {
        let line = raw.trim_end_matches('\r');
        let kind = classify_line(line, &raw_lines, i);

        if needs_separator(kind, prev_non_blank) {
            out.push(separator());
        }

        out.push(render_line(kind, line));

        if kind != Kind::Blank {
            prev_non_blank = kind;
        }
    }

    Text::from(out)
}

fn needs_separator(kind: Kind, prev: Kind) -> bool {
    match kind {
        Kind::Separator => false,
        Kind::Attribution => !matches!(prev, Kind::Attribution | Kind::Separator | Kind::Blank),
        Kind::Header => {
            !matches!(prev, Kind::Header | Kind::Attribution | Kind::Separator | Kind::Blank)
        }
        Kind::Quoted => !matches!(prev, Kind::Quoted | Kind::Attribution | Kind::Separator | Kind::Blank),
        _ => false,
    }
}

fn render_line(kind: Kind, line: &str) -> Line<'static> {
    match kind {
        Kind::Separator => separator(),
        Kind::Attribution => Line::from(Span::styled(
            sanitize_line(line),
            Style::default().add_modifier(Modifier::DIM | Modifier::ITALIC),
        )),
        Kind::Header => header(line),
        Kind::Quoted => {
            let s = sanitize_quoted_line(line);
            if s.trim().is_empty() {
                Line::from("")
            } else {
                Line::from(Span::styled(s, Style::default().add_modifier(Modifier::DIM)))
            }
        }
        Kind::Blank => Line::from(""),
        Kind::Normal => Line::from(Span::raw(sanitize_line(line))),
    }
}

fn header(line: &str) -> Line<'static> {
    let split = line
        .find(" :")
        .filter(|&p| is_header(&line[..p + 1]))
        .map(|p| p + 2)
        .or_else(|| {
            line.find(':')
                .filter(|&p| is_header(&line[..p + 1]))
                .map(|p| p + 1)
        });
    match split {
        Some(after) => Line::from(vec![
            Span::styled(
                sanitize_line(line[..after].trim_end()),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
            ),
            Span::raw(": "),
            Span::raw(sanitize_line(line[after..].trim_start())),
        ]),
        None => Line::from(Span::raw(sanitize_line(line))),
    }
}

fn separator() -> Line<'static> {
    Line::from(Span::styled(
        "─".repeat(72),
        Style::default().add_modifier(Modifier::DIM),
    ))
}
