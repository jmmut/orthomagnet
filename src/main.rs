use macroquad::prelude::*;
use orthomagnet::remote_player::{connect, serve, Command};
use orthomagnet::scenes::menu::Player;
use orthomagnet::scenes::{game, menu, server_waiting};
use orthomagnet::AnyError;
use std::sync::mpsc::{Receiver, Sender};

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
    let Some(player) = menu::scene().await else {
        return Ok(());
    };
    let mut from_client: Option<Receiver<Command>> = None;
    let mut to_client: Option<Sender<Command>> = None;
    let mut from_server: Option<Receiver<Command>> = None;
    let mut to_server: Option<Sender<Command>> = None;
    match player {
        Player::Local => {}
        Player::Server => {
            let (from_client_, to_client_) = serve();
            from_client = Some(from_client_);
            to_client = Some(to_client_);
            let should_continue =
                server_waiting::scene(from_client.as_mut().unwrap(), to_client.as_mut().unwrap())
                    .await;
            if !should_continue {
                return Ok(());
            }
        }
        Player::Client => {
            let (from_server_, to_server_) = connect();
            from_server = Some(from_server_);
            to_server = Some(to_server_);
        }
    }
    game::scene(player, from_client, to_client, from_server, to_server).await?; // TODO: can I just pass non-option from/to remote?
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
