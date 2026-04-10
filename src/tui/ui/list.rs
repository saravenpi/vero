use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{List, ListState},
    Frame,
};

use super::theme::PRIMARY_COLOR;

pub(crate) fn selection_style(is_selected: bool) -> (Color, Color, Modifier) {
    if is_selected {
        (
            PRIMARY_COLOR,
            Color::Reset,
            Modifier::BOLD | Modifier::REVERSED,
        )
    } else {
        (Color::Reset, Color::Reset, Modifier::empty())
    }
}

pub(crate) fn render_stateful_list(
    frame: &mut Frame,
    area: Rect,
    list: List<'_>,
    selected: &mut usize,
    offset: &mut usize,
) {
    let list = list.highlight_style(Style::default());
    let mut state = ListState::default()
        .with_offset(*offset)
        .with_selected(Some(*selected));

    frame.render_stateful_widget(list, area, &mut state);

    *offset = state.offset();
    if let Some(current) = state.selected() {
        *selected = current;
    }
}
