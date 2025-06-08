use crate::{choose_font_size, render_button_flat, FONT, STYLE};
use juquad::widgets::anchor::Anchor;
use juquad::widgets::button::Button;
use juquad::widgets::button_group::ButtonGroup;
use macroquad::color::GRAY;
use macroquad::input::{is_key_pressed, KeyCode};
use macroquad::prelude::{clear_background, next_frame, screen_height, screen_width};

#[derive(Debug)]
pub enum Player {
    Local,
    Server,
    Client,
}

pub async fn scene() -> Option<Player> {
    let mut width = screen_width();
    let mut height = screen_height();
    let (mut _font_size, mut buttons) = reset(width, height);
    loop {
        let new_width = screen_width();
        let new_height = screen_height();
        if new_width != width || new_height != height {
            width = new_width;
            height = new_height;
            (_font_size, buttons) = reset(width, height);
        }
        if is_key_pressed(KeyCode::Escape) || buttons.exit.interact().is_clicked() {
            return None;
        }
        if buttons.local.interact().is_clicked() {
            return Some(Player::Local);
        }
        if buttons.serve.interact().is_clicked() {
            return Some(Player::Server);
        }
        if buttons.connect.interact().is_clicked() {
            return Some(Player::Client);
        }
        clear_background(GRAY);
        buttons.render();
        next_frame().await
    }
}

fn reset(width: f32, height: f32) -> (f32, Buttons) {
    let font_size = choose_font_size(width, height) * 2.0;
    let buttons = create_button_group(font_size, width, height);
    (font_size, buttons)
}

pub struct Buttons {
    pub local: Button,
    pub connect: Button,
    pub serve: Button,
    pub exit: Button,
}
impl Buttons {
    pub fn render(&self) {
        self.local.render(&STYLE);
        self.connect.render(&STYLE);
        self.serve.render(&STYLE);
        self.exit.render(&STYLE);
    }
}

fn create_button_group(font_size: f32, width: f32, height: f32) -> Buttons {
    let mut button_group = ButtonGroup::new_with_font(
        font_size,
        unsafe { FONT },
        Anchor::top_center(width * 0.5, height * 0.25),
    );
    button_group.render = render_button_flat;
    let [local, connect, serve, exit] =
        button_group.create(["Local game", "Connect to server", "Serve game", "Exit"]);
    Buttons {
        local,
        connect,
        serve,
        exit,
    }
}
