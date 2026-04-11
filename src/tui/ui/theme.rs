use ratatui::style::{Color, Modifier, Style};

pub(crate) const PRIMARY_COLOR: Color = Color::Cyan;
pub(crate) const SUCCESS_COLOR: Color = Color::Green;
pub(crate) const ERROR_COLOR: Color = Color::Red;

pub(crate) fn muted_text_style() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}
