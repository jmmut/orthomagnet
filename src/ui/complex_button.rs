use crate::ui::button_trait::{ButtonBase, ButtonTrait};
use crate::{new_text_alt_font, render_button_base};
use juquad::input::input_macroquad::InputMacroquad;
use juquad::input::input_trait::InputTrait;
use juquad::widgets::anchor::{Anchor, Vertical};
use juquad::widgets::button::{Interaction, Style};
use juquad::widgets::text::TextRect;
use juquad::widgets::Widget;
use macroquad::math::{vec2, Vec2};
use macroquad::prelude::{draw_texture_ex, DrawTextureParams, Rect, Texture2D};

pub struct ComplexButton {
    pub rect: Rect,
    pub icon_rect: Rect,
    pub icon_at_rest: Vec<Texture2D>,
    // pub icon_hovered: Vec<Texture2D>,
    // pub icon_pressed: Vec<Texture2D>,
    pub text: TextRect,
    pub base: ButtonBase,
}

impl ComplexButton {
    pub fn new(
        anchor: Anchor,
        textures: Vec<Texture2D>,
        texture_size: Vec2,
        text: &str,
        font_size: f32,
    ) -> Self {
        let mut text_anchor = anchor;
        text_anchor.offset(texture_size.x, 0.0);
        let text = new_text_alt_font(text, text_anchor, font_size);
        let icon_anchor = Anchor::leftwards(text.rect(), Vertical::Center, -texture_size.x * 0.35);
        let icon_rect = icon_anchor.get_rect(texture_size);
        let rect = anchor.get_rect(text.rect().size() + vec2(icon_rect.w, 0.0));
        Self {
            rect,
            icon_rect,
            icon_at_rest: textures,
            text,
            base: ButtonBase {
                interaction: Interaction::None,
                input: Box::new(InputMacroquad),
            },
        }
    }
}

impl ButtonTrait for ComplexButton {
    fn render(&self, style: &Style) {
        let style = render_button_base(self.interaction(), self.rect, style);
        for texture in self.icon_at_rest.clone() {
            draw_texture_ex(
                texture,
                self.icon_rect.x,
                self.icon_rect.y,
                style.text_color,
                DrawTextureParams {
                    dest_size: Some(self.icon_rect.size()),
                    ..Default::default()
                },
            );
            // if unsafe { SHADOWS } || true {
            //     draw_texture_ex(
            //         texture,
            //         self.icon_rect.x + 1.0,
            //         self.icon_rect.y + 1.0,
            //         darken(style.text_color),
            //         DrawTextureParams {
            //             dest_size: Some(self.icon_rect.size()),
            //             ..Default::default()
            //         },
            //     );
            // }
        }
        self.text.render_text(style.text_color);
    }
    fn interaction(&self) -> Interaction {
        self.base.interaction
    }
    fn interaction_mut(&mut self) -> &mut Interaction {
        &mut self.base.interaction
    }
    fn input(&self) -> &Box<dyn InputTrait> {
        &self.base.input
    }
}
impl Widget for ComplexButton {
    fn rect(&self) -> Rect {
        self.rect
    }
    fn rect_mut(&mut self) -> &mut Rect {
        &mut self.rect
    }
}
