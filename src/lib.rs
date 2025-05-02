use juquad::draw::{draw_rect, draw_rect_lines};
use juquad::widgets::button::{Interaction, Style};
use juquad::widgets::text::TextRect;

pub type AnyError = Box<dyn std::error::Error>;

pub fn choose_font_size(width: f32, height: f32) -> f32 {
    const FONT_SIZE: f32 = 16.0;
    let min_side = height.min(width * 16.0 / 9.0);
    FONT_SIZE
        * if min_side < 1200.0 {
            1.0
        } else if min_side < 1800.0 {
            1.5
        } else {
            2.0
        }
}

pub fn render_button_flat(interaction: Interaction, text_rect: &TextRect, style: &Style) {
    let (bg_color, text_color, border_color) = match interaction {
        Interaction::Clicked | Interaction::Pressing => (
            style.bg_color.pressed,
            style.text_color.pressed,
            style.border_color.pressed,
        ),
        Interaction::Hovered => (
            style.bg_color.hovered,
            style.text_color.hovered,
            style.border_color.hovered,
        ),
        Interaction::None => (
            style.bg_color.at_rest,
            style.text_color.at_rest,
            style.border_color.at_rest,
        ),
    };
    let rect = text_rect.rect;
    draw_rect(rect, bg_color);
    draw_rect_lines(rect, 2.0, border_color);
    text_rect.render_text(text_color);
}
