use juquad::draw::{draw_rect, draw_rect_lines};
use juquad::input::input_macroquad::InputMacroquad;
use juquad::widgets::anchor::Anchor;
use juquad::widgets::button::{Button};
use juquad::widgets::text::{draw_text_rect_generic, TextRect};
use juquad::widgets::{Interaction, StateStyle, Style, Widget};
use macroquad::color::{BLACK, DARKGRAY, LIGHTGRAY, WHITE};
use macroquad::prelude::{load_ttf_font_from_bytes, measure_text, Color, Font, Rect, TextParams};

pub mod scenes {
    pub mod game;
    pub mod loading;
    pub mod menu;
    pub mod server_waiting;
}
pub mod ui {
    pub mod button_trait;
    pub mod complex_button;
    pub mod counter;
}
pub mod board;
pub mod remote_player;

pub type AnyError = Box<dyn std::error::Error>;

pub const FONT_BYTES: &[u8] = include_bytes!("../assets/Saira-Regular.ttf");
pub const BASE_FONT_SIZE: f32 = 16.0; // prefer using choose_font_size()
pub static mut FONT: Option<Font> = None;

pub const STYLE: Style = Style {
    at_rest: StateStyle {
        text_color: DARKGRAY,
        bg_color: LIGHTGRAY,
        border_color: DARKGRAY,
    },
    hovered: StateStyle {
        text_color: BLACK,
        bg_color: WHITE,
        border_color: BLACK,
    },
    pressed: StateStyle {
        text_color: LIGHTGRAY,
        bg_color: DARKGRAY,
        border_color: LIGHTGRAY,
    },
};

pub fn choose_font_size(width: f32, height: f32) -> f32 {
    let min_side = height.min(width * 16.0 / 9.0);
    BASE_FONT_SIZE
        * if min_side < 1200.0 {
            1.0
        } else if min_side < 1800.0 {
            1.5
        } else {
            2.0
        }
}

fn setup_font() -> Result<(), AnyError> {
    let font = load_ttf_font_from_bytes(FONT_BYTES)?;
    unsafe {
        FONT = Some(font);
    };
    Ok(())
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
    Button::new_from_text_rect_generic(text_rect, Box::new(InputMacroquad))
}
pub fn new_text_alt_font(text: &str, position: Anchor, font_size: f32) -> TextRect {
    TextRect::new_generic(
        text,
        position,
        font_size,
        unsafe { FONT },
        measure_text,
    )
}
static mut SHADOWS: bool = false;

pub fn draw_text_shadow(text_rect: &TextRect, style: &StateStyle) {
    draw_text_rect_generic(text_rect, style, draw_text_shadow_deconstructed);
}
pub fn draw_text_shadow_deconstructed(
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    style: &StateStyle,
    font: Option<Font>,
) {
    let color = style.text_color;
    let color_shadow = darken(color);
    if let Some(font) = font {
        let mut params = TextParams {
            font,
            font_size: font_size as u16,
            color,
            ..TextParams::default()
        };
        params.color = color_shadow;
        if unsafe { SHADOWS } {
            macroquad::text::draw_text_ex(text, x + 1.0, y + 1.0, params);
        }
        params.color = color;
        macroquad::text::draw_text_ex(text, x, y, params);
    } else {
        if unsafe { SHADOWS } {
            macroquad::text::draw_text(text, x + 1.0, y + 1.0, font_size, color_shadow)
        }
        macroquad::text::draw_text(text, x, y, font_size, color);
    }
}
pub fn darken(color: Color) -> Color {
    Color::new(
        (color.r + 0.2).min(1.0),
        (color.g + 0.2).min(1.0),
        (color.b + 0.2).min(1.0),
        (color.a - 0.5).max(0.0),
    )
}
pub fn invert(color: Color) -> Color {
    // Color::new(
    //     1.0 - color.r,
    //     1.0 - color.g,
    //     1.0 - color.b,
    //     color.a,
    // )
    Color::new(1.0, 1.0, 1.0, color.a)
}

pub fn render_button_flat(button: &Button, style: &Style) {
    button.render(style, render_button_flat_deconstructed);
}
pub fn render_button_flat_deconstructed(interaction: Interaction, text_rect: &TextRect, style: &Style) {
    let state_style = render_button_base(interaction, text_rect.rect(), style);
    draw_text_rect_generic(text_rect, state_style, draw_text_shadow_deconstructed)
}
pub fn render_button_base(interaction: Interaction, rect: Rect, style: &Style) -> &StateStyle {
    let state_style = style.choose(interaction);
    draw_rect(rect, state_style.bg_color);
    // let smaller = Rect::new(rect.x+1.0, rect.y, rect.w - 2.0, rect.h);
    // draw_rect(smaller, single_style.bg_color);
    // draw_line(rect.x+1.0, rect.y + 1.0, rect.x+1.0, rect.bottom() - 1.0,  1.0, single_style.bg_color);
    // draw_line(rect.right(), rect.y + 1.0, rect.right(), rect.bottom() - 1.0,  1.0, single_style.bg_color);
    draw_rect_lines(rect, 2.0, state_style.border_color);
    state_style
}
