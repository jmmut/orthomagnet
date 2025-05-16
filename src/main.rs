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
        game::scene(Player::Local, None, None).await?;
    } else {
        if let Some(player) = menu::scene().await {
            let (from_remote, to_remote) = match player {
                Player::Local => (None, None),
                Player::Server => {
                    let (from_client_, to_client_) = serve();
                    if !server_waiting::scene(&from_client_, &to_client_).await {
                        return Ok(());
                    }
                    (Some(from_client_), Some(to_client_))
                }
                Player::Client => {
                    let (from_server_, to_server_) = connect();
                    (Some(from_server_), Some(to_server_))
                }
            };

            game::scene(player, from_remote, to_remote).await?; // TODO: can I just pass non-option from/to remote?
        }
    }
    Ok(())
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
