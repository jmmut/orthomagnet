use juquad::draw::{draw_rect, draw_rect_lines};
use juquad::input::input_macroquad::InputMacroquad;
use juquad::widgets::anchor::Anchor;
use juquad::widgets::button::{Button, Interaction, InteractionStyle, Style};
use juquad::widgets::text::{draw_text, TextRect};
use macroquad::color::{BLACK, DARKGRAY, LIGHTGRAY, WHITE};
use macroquad::prelude::{measure_text, Font};

pub mod scenes {
    pub mod game;
    pub mod menu;
    pub mod server_waiting;
}
pub mod board;
pub mod counter;
pub mod remote_player;

pub type AnyError = Box<dyn std::error::Error>;

pub const FONT_BYTES: &[u8] = include_bytes!("../assets/Saira-Regular.ttf");
pub static mut FONT: Option<Font> = None;

pub const STYLE: Style = Style {
    bg_color: InteractionStyle {
        at_rest: LIGHTGRAY,
        hovered: WHITE,
        pressed: DARKGRAY,
    },
    text_color: InteractionStyle {
        at_rest: DARKGRAY,
        hovered: BLACK,
        pressed: LIGHTGRAY,
    },
    border_color: InteractionStyle {
        at_rest: DARKGRAY,
        hovered: BLACK,
        pressed: LIGHTGRAY,
    },
};

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

pub fn new_button(text: &str, position: Anchor, font_size: f32) -> Button {
    let text_rect = TextRect::new(text, position, font_size);
    new_button_from_text_rect(text_rect)
}

pub fn new_button_alt_font(text: &str, position: Anchor, font_size: f32) -> Button {
    let text_rect = new_text_alt_font(text, position, font_size);
    // text_rect.pad.y = font_size * 0.6;
    new_button_from_text_rect(text_rect)
}

pub fn new_button_from_text_rect(text_rect: TextRect) -> Button {
    Button::new_from_text_rect_generic(text_rect, render_button_flat, Box::new(InputMacroquad))
}
pub fn new_text_alt_font(text: &str, position: Anchor, font_size: f32) -> TextRect {
    TextRect::new_generic(
        text,
        position,
        font_size,
        unsafe { FONT },
        measure_text,
        draw_text,
    )
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
