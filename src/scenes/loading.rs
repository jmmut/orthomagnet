use crate::{choose_font_size, setup_font, AnyError};
use juquad::texture_loader::TextureLoader;
use juquad::widgets::anchor::Anchor;
use juquad::widgets::button::{InteractionStyle, Style};
use juquad::widgets::text::TextRect;
use macroquad::color::WHITE;
use macroquad::prelude::{next_frame, screen_height, screen_width, FilterMode, Texture2D};

const LOADING_STYLE: Style = Style {
    text_color: InteractionStyle {
        at_rest: WHITE,
        hovered: WHITE,
        pressed: WHITE,
    },
    ..Style::new()
};
pub struct Textures {
    pub restart: Texture2D,
    pub undo: Texture2D,
}
pub async fn scene() -> Result<Textures, AnyError> {
    setup_font()?;
    let mut loader = TextureLoader::new(&[
        "assets/images/restart_shadow.png",
        "assets/images/undo_shadow.png",
        // "assets/images/restart.png",
        // "assets/images/undo.png",
    ]);

    let mut width = screen_width();
    let mut height = screen_height();
    let mut text = reset(width, height);
    loop {
        if let Some(mut textures) = loader.get_textures()? {
            for texture in &mut textures {
                texture.set_filter(FilterMode::Nearest);
            }
            return Ok(Textures {
                restart: textures[0],
                undo: textures[1],
            });
        }
        let new_width = screen_width();
        let new_height = screen_height();
        if new_width != width || new_height != height {
            width = new_width;
            height = new_height;
            text = reset(width, height);
        }
        text.render(&LOADING_STYLE);
        next_frame().await;
    }
}

fn reset(sw: f32, sh: f32) -> TextRect {
    let anchor = Anchor::center(sw * 0.5, sh * 0.5);
    TextRect::new("LOADING...", anchor, 2.0 * choose_font_size(sw, sh))
}
