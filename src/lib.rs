use juquad::draw::{draw_rect, draw_rect_lines};
use juquad::input::input_macroquad::InputMacroquad;
use juquad::widgets::anchor::Anchor;
use juquad::widgets::button::{Button, Interaction, InteractionStyle, Style};
use juquad::widgets::text::TextRect;
use macroquad::color::{BLACK, DARKGRAY, LIGHTGRAY, WHITE};

pub mod scenes {
    pub mod game;
    pub mod menu;
    pub mod server_waiting;
}
pub mod board;
pub mod counter;
pub mod remote_player;

pub type AnyError = Box<dyn std::error::Error>;

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
    Button::new_from_text_rect_generic(
        TextRect::new(text, position, font_size),
        render_button_flat,
        Box::new(InputMacroquad),
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
