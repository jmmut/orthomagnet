use std::future::Future;
use macroquad::prelude::*;
use orthomagnet::remote_player::{connect, serve};
use orthomagnet::scenes::menu::Player;
use orthomagnet::scenes::{game, menu, server_waiting};
use orthomagnet::{AnyError, FONT, FONT_BYTES};

const DEFAULT_WINDOW_WIDTH: i32 = 450;
const DEFAULT_WINDOW_HEIGHT: i32 = 800;
const DEFAULT_WINDOW_TITLE: &str = "orthomagnet";

#[macroquad::main(window_conf)]
async fn main() {
    if let Err(e) = try_main().await {
        println!("Server thread error: {}", e);
    }
}

async fn try_main() -> Result<(), AnyError> {
    setup_font()?;

    let enable_networking = false;
    if !enable_networking {
        game::scene(Player::Local, None, None).await
    } else {
        if let Some(player) = menu::scene().await {
            match player {
                Player::Local => game::scene(player, None, None).await,
                Player::Server => {
                    let (from_client_, to_client_) = serve();
                    if server_waiting::scene(&from_client_, &to_client_).await {
                        game::scene(player, Some(from_client_), Some(to_client_)).await
                    } else {
                        Ok(())
                    }
                }
                Player::Client => {
                    let (from_server_, to_server_) = connect();
                    game::scene(player, Some(from_server_), Some(to_server_)).await
                }
            }
        } else {
            Ok(())
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: DEFAULT_WINDOW_TITLE.to_owned(),
        window_width: DEFAULT_WINDOW_WIDTH,
        window_height: DEFAULT_WINDOW_HEIGHT,
        high_dpi: true,
        ..Default::default()
    }
}

fn setup_font() -> Result<(), AnyError> {
    let font = load_ttf_font_from_bytes(FONT_BYTES)?;
    unsafe {
        FONT = Some(font);
    };
    Ok(())
}
