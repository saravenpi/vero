use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Text,
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthChar;

use super::theme::PRIMARY_COLOR;

pub(crate) fn render<'t>(frame: &mut Frame, area: Rect, scroll_offset: &mut usize, body: Text<'t>) {
    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let body_text_area = body_layout[0];
    let scrollbar_area = body_layout[1];

    let body_line_count = rendered_text_line_count(&body, body_text_area.width);
    let viewport_height = body_text_area.height as usize;
    let max_scroll = body_line_count.saturating_sub(viewport_height);
    *scroll_offset = (*scroll_offset).min(max_scroll);

    let para = Paragraph::new(body)
        .wrap(Wrap { trim: false })
        .scroll(((*scroll_offset).min(u16::MAX as usize) as u16, 0));
    frame.render_widget(para, body_text_area);

    if scrollbar_area.width == 0 || scrollbar_area.height == 0 {
        return;
    }

    let scrollbar_content_length = if max_scroll == 0 { 1 } else { max_scroll + 1 };
    let mut scrollbar_state =
        ScrollbarState::new(scrollbar_content_length).position(*scroll_offset);

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .track_style(Style::default().add_modifier(Modifier::DIM))
            .thumb_symbol("█")
            .thumb_style(Style::default().fg(PRIMARY_COLOR)),
        scrollbar_area,
        &mut scrollbar_state,
    );
}

fn rendered_text_line_count(text: &Text<'_>, width: u16) -> usize {
    let width = width as usize;
    if width == 0 {
        return 0;
    }

    text.lines
        .iter()
        .map(|line| {
            let content: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            if content.is_empty() {
                1
            } else {
                wrapped_line_count(&content, width)
            }
        })
        .sum()
}

fn wrapped_line_count(line: &str, width: usize) -> usize {
    if line.is_empty() {
        return 1;
    }

    let mut lines = 1;
    let mut current_width = 0;
    let mut run = String::new();
    let mut run_is_whitespace = None;

    for ch in line.chars() {
        let is_whitespace = ch.is_whitespace();
        if run_is_whitespace == Some(is_whitespace) || run_is_whitespace.is_none() {
            run.push(ch);
            run_is_whitespace = Some(is_whitespace);
            continue;
        }

        push_run(
            run.as_str(),
            run_is_whitespace.unwrap_or(false),
            width,
            &mut lines,
            &mut current_width,
        );
        run.clear();
        run.push(ch);
        run_is_whitespace = Some(is_whitespace);
    }

    if !run.is_empty() {
        push_run(
            run.as_str(),
            run_is_whitespace.unwrap_or(false),
            width,
            &mut lines,
            &mut current_width,
        );
    }

    lines
}

fn push_run(
    run: &str,
    is_whitespace: bool,
    width: usize,
    lines: &mut usize,
    current_width: &mut usize,
) {
    let run_width = run
        .chars()
        .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(0))
        .sum::<usize>();

    if run_width == 0 {
        return;
    }

    if !is_whitespace && *current_width > 0 && *current_width + run_width > width {
        *lines += 1;
        *current_width = 0;
    }

    if run_width <= width.saturating_sub(*current_width) {
        *current_width += run_width;
        return;
    }

    for ch in run.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if ch_width == 0 {
            continue;
        }

        if *current_width + ch_width > width {
            *lines += 1;
            *current_width = 0;
        }

        *current_width += ch_width;
    }
}
