use crate::remote_player::Command;
use crate::{choose_font_size, new_button, STYLE};
use juquad::widgets::anchor::Anchor;
use juquad::widgets::button::Button;
use juquad::widgets::text::TextRect;
use macroquad::input::{is_key_pressed, KeyCode};
use macroquad::miniquad::date::now;
use macroquad::prelude::{clear_background, next_frame, screen_height, screen_width, BLACK, GRAY};
use std::sync::mpsc::{Receiver, Sender};

pub async fn scene(from_client: &Receiver<Command>, _to_client: &Sender<Command>) -> bool {
    let text_color = BLACK;
    let mut width = screen_width();
    let mut height = screen_height();
    let (mut text, mut animation, mut exit) = reset(width, height);
    loop {
        let new_width = screen_width();
        let new_height = screen_height();
        if new_width != width || new_height != height {
            width = new_width;
            height = new_height;
            (text, animation, exit) = reset(width, height);
        }

        if let Ok(Command::Connected) = from_client.try_recv() {
            // TODO: send ack to client?
            return true;
        }
        if is_key_pressed(KeyCode::Escape) || exit.interact().is_clicked() {
            return false;
        }

        clear_background(GRAY);
        text.render_text(text_color);
        let now_seconds = now();
        animation[now_seconds.trunc() as usize % 3].render_text(text_color);
        exit.render(&STYLE);

        next_frame().await
    }
}

fn reset(width: f32, height: f32) -> (TextRect, [TextRect; 3], Button) {
    let font_size = choose_font_size(width, height) * 1.5;
    let anchor = Anchor::top_center(width * 0.5, height * 0.25);
    let text_rect = TextRect::new("Waiting for connections", anchor, font_size);
    let anchor = Anchor::top_center(screen_width() * 0.5, text_rect.rect.bottom());

    let animation = [
        TextRect::new(".", anchor, font_size),
        TextRect::new("..", anchor, font_size),
        TextRect::new("...", anchor, font_size),
    ];
    let anchor = Anchor::top_center(screen_width() * 0.5, animation[0].rect.bottom());
    let exit = new_button("exit", anchor, font_size);
    (text_rect, animation, exit)
}
