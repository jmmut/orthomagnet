use juquad::draw::{draw_rect, draw_rect_lines};
use juquad::input::input_macroquad::InputMacroquad;
use juquad::widgets::anchor::Anchor;
use juquad::widgets::button::{Button, Interaction, InteractionStyle, Style};
use juquad::widgets::text::TextRect;
use macroquad::prelude::*;
use orthomagnet::remote_player::{connect, serve, Command};
use orthomagnet::scenes::menu::{menu_scene, Player};
use orthomagnet::scenes::server_waiting::server_waiting_scene;
use orthomagnet::{choose_font_size, new_button, render_button_flat, AnyError, STYLE};
use std::sync::mpsc::{Receiver, Sender};

const DEFAULT_WINDOW_WIDTH: i32 = 450;
const DEFAULT_WINDOW_HEIGHT: i32 = 800;
const DEFAULT_WINDOW_TITLE: &str = "orthomagnet";

const BOARD_TOP_COEF: f32 = 0.12;
const BOARD_LEFT_COEF: f32 = 0.15;
const BOARD_WIDTH_COEF: f32 = 1.0 - 2.0 * BOARD_LEFT_COEF;
const BOARD_HEIGHT_COEF: f32 = 0.6;

const WHITE_HINT: Color = Color::new(1.0, 1.0, 1.0, 0.3);
const BLACK_HINT: Color = Color::new(0.0, 0.0, 0.0, 0.3);
const WHITE_FULL: Color = Color::new(1.0, 1.0, 1.0, 0.7);
const BLACK_FULL: Color = Color::new(0.0, 0.0, 0.0, 0.7);
const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);

#[derive(Copy, Clone, PartialEq, Debug)]
enum Team {
    Empty,
    White,
    Black,
}
impl Team {
    fn toggle(&self) -> Team {
        match self {
            Team::White => Team::Black,
            Team::Black => Team::White,
            _ => panic!("can not toggle a Team::Empty"),
        }
    }
    fn choose<T>(&self, if_empty: T, if_white: T, if_black: T) -> T {
        match self {
            Team::Empty => if_empty,
            Team::White => if_white,
            Team::Black => if_black,
        }
    }
}

pub struct Counter {
    pub vertical_pad: f32,
    pub increase: Button,
    pub counter: TextRect,
    pub decrease: Button,
    pub rect: Rect,
}
impl Counter {
    pub fn new(count: i32, position: Anchor, vertical_pad: f32, font_size: f32) -> Self {
        let tmp_anchor = Anchor::top_left(0.0, 0.0);
        let mut increase = new_button("+", tmp_anchor, font_size);
        let mut counter = TextRect::new(
            count.to_string().as_str(),
            from_below(increase.rect(), 0.0, vertical_pad),
            font_size,
        );
        let mut decrease = new_button(
            "-",
            Anchor::from_below(counter.rect, 0.0, vertical_pad),
            font_size,
        );
        let mut rect = increase
            .rect()
            .combine_with(counter.rect)
            .combine_with(decrease.rect());

        let diff = position.get_top_left_pixel(rect.size());
        increase.text_rect.rect = increase.text_rect.rect.offset(diff);
        counter.rect = counter.rect.offset(diff);
        decrease.text_rect.rect = decrease.text_rect.rect.offset(diff);
        rect = rect.offset(diff);
        Self {
            vertical_pad,
            increase,
            counter,
            decrease,
            rect,
        }
    }

    pub fn update(&mut self, new_count: i32) {
        self.counter = TextRect::new(
            new_count.to_string().as_str(),
            from_below(self.increase.rect(), 0.0, self.vertical_pad),
            self.counter.font_size,
        )
    }
    pub fn render(&self, style: &Style) {
        draw_rect(
            self.increase.rect().combine_with(self.decrease.rect()),
            DARKGRAY,
        );
        self.increase.render(style);
        self.counter.render_text(LIGHTGRAY);
        self.decrease.render(style);
    }
}

fn from_below(other: Rect, x_diff: f32, y_diff: f32) -> Anchor {
    Anchor::top_left(other.x + x_diff, other.y + other.h + y_diff)
}
pub struct Buttons {
    pub restart: Button,
    pub undo: Button,
    pub rows: Counter,
    pub columns: Counter,
}

impl Buttons {
    pub fn new(screen_width: f32, screen_height: f32, row_count: i32, column_count: i32) -> Self {
        let font_size = choose_font_size(screen_width, screen_height);
        let left_pad = 16.0;
        let counter_inner_pad = 0.0;
        let left = (BOARD_LEFT_COEF * screen_width).round();
        let bottom = (screen_height * (1.0 - BOARD_TOP_COEF)
            + score_font_size(screen_width, screen_height))
        .round();
        let undo = new_button("Undo", Anchor::bottom_left(left, bottom), font_size * 1.5);
        let restart_anchor = Anchor::bottom_left(undo.rect().x, undo.rect().y - undo.rect().h);
        let restart = new_button("Restart", restart_anchor, font_size * 1.5);

        let anchor_columns =
            Anchor::bottom_right(((1.0 - BOARD_LEFT_COEF) * screen_width).round(), bottom);
        let columns = Counter::new(
            column_count,
            anchor_columns,
            counter_inner_pad,
            font_size * 1.5,
        );
        let anchor_rows = Anchor::top_right(columns.rect.x - left_pad * 0.5, columns.rect.y);
        let rows = Counter::new(row_count, anchor_rows, counter_inner_pad, font_size * 1.5);
        Self {
            restart,
            undo,
            rows,
            columns,
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    if let Err(e) = try_main().await {
        println!("Server thread error: {}", e);
    }
}
async fn try_main() -> Result<(), AnyError> {
    let Some(player) = menu_scene().await else {
        return Ok(());
    };
    let mut turn = Team::White;
    let mut size_rows = 7;
    let mut size_columns = 5;
    let mut board = new_board(size_rows, size_columns);
    let mut board_history = Vec::new();
    let mut prev_sw = screen_width();
    let mut prev_sh = screen_height();
    let mut buttons = Buttons::new(prev_sw, prev_sh, size_rows, size_columns);
    let mut remote_mouse = None;
    let mut previous_mouse_tile = None;
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
                server_waiting_scene(from_client.as_mut().unwrap(), to_client.as_mut().unwrap())
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
    loop {
        if screen_height() != prev_sh || screen_width() != prev_sw {
            prev_sw = screen_width();
            prev_sh = screen_height();
            buttons = Buttons::new(prev_sw, prev_sh, size_rows, size_columns);
        }
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::R) || buttons.restart.interact().is_clicked() {
            board = new_board(size_rows, size_columns);
        }
        if is_key_pressed(KeyCode::Z) && is_key_down(KeyCode::LeftControl)
            || buttons.undo.interact().is_clicked()
        {
            if let Some(b) = board_history.pop() {
                board = b;
                turn = turn.toggle()
            }
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            println!("{}", Vec2::from(mouse_position()));
        }
        maybe_change_size(&mut size_rows, &mut size_columns, &mut buttons, &mut board);
        clear_background(GRAY);
        let sw = screen_width();
        let sh = screen_height();

        let board_rect = Rect::new(
            sw * BOARD_LEFT_COEF,
            sh * BOARD_TOP_COEF,
            sw * BOARD_WIDTH_COEF,
            sh * BOARD_HEIGHT_COEF,
        );
        draw_board_lines(board_rect, size_rows, size_columns);
        match player {
            Player::Local => {
                if let Some(tile) = get_tile(
                    board_rect,
                    (size_rows, size_columns),
                    Vec2::from(mouse_position()),
                ) {
                    let color = turn.choose(TRANSPARENT, WHITE_HINT, BLACK_HINT);
                    draw_stone(tile, color, board_rect, size_rows, size_columns);
                    if is_mouse_button_released(MouseButton::Left) {
                        board_history.push(board.clone());
                        let previous_turn = turn;
                        try_put_stone(&mut turn, &mut board, tile);
                        if turn == previous_turn {
                            board_history.pop();
                        }
                    }
                }
            }
            Player::Server => {
                let remote_color = BLACK_HINT;
                let local_color = WHITE_HINT;
                let local_team = Team::White;
                let to_remote = to_client.as_ref().unwrap();
                let from_remote = from_client.as_mut().unwrap();
                update_mouses(
                    &mut turn,
                    size_rows,
                    size_columns,
                    &mut board,
                    &mut board_history,
                    &mut remote_mouse,
                    &mut previous_mouse_tile,
                    board_rect,
                    remote_color,
                    local_color,
                    local_team,
                    to_remote,
                    from_remote,
                )?;
            }
            Player::Client => {
                let remote_color = WHITE_HINT;
                let local_color = BLACK_HINT;
                let local_team = Team::Black;
                let to_remote = to_server.as_ref().unwrap();
                let from_remote = from_server.as_mut().unwrap();
                update_mouses(
                    &mut turn,
                    size_rows,
                    size_columns,
                    &mut board,
                    &mut board_history,
                    &mut remote_mouse,
                    &mut previous_mouse_tile,
                    board_rect,
                    remote_color,
                    local_color,
                    local_team,
                    to_remote,
                    from_remote,
                )?;
            }
        }
        draw_stones(&board, board_rect, size_rows, size_columns);
        draw_score(board_rect, &board);
        draw_instructions(&buttons);
        draw_size(&buttons);
        next_frame().await
    }
    Ok(())
}

fn update_mouses(
    turn: &mut Team,
    size_rows: i32,
    size_columns: i32,
    board: &mut Vec<Vec<Team>>,
    board_history: &mut Vec<Vec<Vec<Team>>>,
    remote_mouse: &mut Option<IVec2>,
    previous_mouse_tile: &mut Option<IVec2>,
    board_rect: Rect,
    remote_color: Color,
    local_color: Color,
    local_team: Team,
    to_remote: &Sender<Command>,
    from_remote: &mut Receiver<Command>,
) -> Result<(), AnyError> {
    update_remote_mouse(remote_mouse, from_remote);
    if let Some(tile) = remote_mouse.as_ref() {
        draw_stone(*tile, remote_color, board_rect, size_rows, size_columns);
    }
    let new_tile_opt = update_local_mouse(
        turn,
        local_team,
        size_rows,
        size_columns,
        board,
        board_history,
        board_rect,
        local_color,
    );
    send_local_mouse_update(previous_mouse_tile, to_remote, new_tile_opt)?;
    Ok(())
}

fn update_remote_mouse(remote_mouse: &mut Option<IVec2>, from_remote: &mut Receiver<Command>) {
    while let Ok(command) = from_remote.try_recv() {
        match command {
            Command::StoneHover { x, y } => {
                *remote_mouse = Some(IVec2::new(x, y));
            }
            Command::StopStoneHover => {
                *remote_mouse = None;
            }
            Command::Connected => unreachable!(),
        }
    }
}

fn update_local_mouse(
    turn: &mut Team,
    local_team: Team,
    size_rows: i32,
    size_columns: i32,
    board: &mut Vec<Vec<Team>>,
    board_history: &mut Vec<Vec<Vec<Team>>>,
    board_rect: Rect,
    local_color: Color,
) -> Option<IVec2> {
    let new_tile = get_tile(
        board_rect,
        (size_rows, size_columns),
        Vec2::from(mouse_position()),
    );
    if let Some(tile) = new_tile {
        draw_stone(tile, local_color, board_rect, size_rows, size_columns);
        if *turn == local_team {
            if is_mouse_button_released(MouseButton::Left) {
                board_history.push(board.clone());
                let previous_turn = *turn;
                try_put_stone(turn, board, tile);
                if *turn == previous_turn {
                    board_history.pop();
                }
            }
        }
    }
    new_tile
}

fn send_local_mouse_update(previous_mouse_tile: &mut Option<IVec2>, to_remote: &Sender<Command>, new_tile_opt: Option<IVec2>) -> Result<(), AnyError> {
    if new_tile_opt != *previous_mouse_tile {
        *previous_mouse_tile = new_tile_opt;
        let command = if let Some(new_tile) = new_tile_opt {
            Command::StoneHover {
                x: new_tile.x,
                y: new_tile.y,
            }
        } else {
            Command::StopStoneHover
        };
        to_remote.send(command)?;
    }
    Ok(())
}

fn maybe_change_size(
    size_rows: &mut i32,
    size_columns: &mut i32,
    buttons: &mut Buttons,
    board: &mut Vec<Vec<Team>>,
) {
    let mut changed_rows = false;
    let mut changed_columns = false;
    if buttons.rows.increase.interact().is_clicked() {
        *size_rows += 1;
        changed_rows = true;
    }
    if buttons.rows.decrease.interact().is_clicked() {
        *size_rows -= 1;
        changed_rows = true;
    }
    if buttons.columns.increase.interact().is_clicked() {
        *size_columns += 1;
        changed_columns = true;
    }
    if buttons.columns.decrease.interact().is_clicked() {
        *size_columns -= 1;
        changed_columns = true;
    }
    if changed_rows {
        buttons.rows.update(*size_rows);
    }
    if changed_columns {
        buttons.columns.update(*size_columns);
    }
    if changed_rows || changed_columns {
        *board = new_board(*size_rows, *size_columns);
    }
}

fn try_put_stone(turn: &mut Team, mut board: &mut Vec<Vec<Team>>, tile: IVec2) {
    let row_count = board[0].len() as i32;
    let column_count = board.len() as i32;
    let clicked = &mut board[tile.x as usize][tile.y as usize];
    if let Team::Empty = clicked {
        *clicked = *turn;
        let side_xp = tile + IVec2::new(1, 0);
        let side_xn = tile + IVec2::new(-1, 0);
        let side_yp = tile + IVec2::new(0, 1);
        let side_yn = tile + IVec2::new(0, -1);
        let mut check_xp = Team::Empty == get_team(&board, side_xp);
        let mut check_xn = Team::Empty == get_team(&board, side_xn);
        let mut check_yp = Team::Empty == get_team(&board, side_yp);
        let mut check_yn = Team::Empty == get_team(&board, side_yn);
        // space for pulling stones in +x
        let opponent = turn.toggle();
        let mut check_dir = |adjacent_to_new_stone: IVec2,
                             diff_new_stone_with_pulled_stone: IVec2,
                             keep_checking: &mut bool| {
            check_direction(
                turn,
                opponent,
                tile,
                adjacent_to_new_stone,
                diff_new_stone_with_pulled_stone,
                &mut board,
                keep_checking,
            );
        };
        for i in 2..row_count {
            check_dir(side_xp, IVec2::new(i, 0), &mut check_xp);
            check_dir(side_xn, IVec2::new(-i, 0), &mut check_xn);
        }
        for i in 2..column_count {
            check_dir(side_yp, IVec2::new(0, i), &mut check_yp);
            check_dir(side_yn, IVec2::new(0, -i), &mut check_yn);
        }
        *turn = turn.toggle();
    }
}

fn check_direction(
    new_stone_color: &mut Team,
    opponent: Team,
    new_stone: IVec2,
    adjacent_to_new_stone: IVec2,
    diff_new_stone_with_pulled_stone: IVec2,
    mut board: &mut &mut Vec<Vec<Team>>,
    keep_checking: &mut bool,
) {
    if *keep_checking {
        let pulled_stone = new_stone + diff_new_stone_with_pulled_stone;
        let pulled_stone_color = get_team(&board, pulled_stone);
        if *new_stone_color == pulled_stone_color {
            *keep_checking = false;
        }
        if opponent == pulled_stone_color {
            *get_team_mut(&mut board, adjacent_to_new_stone) = *new_stone_color;
            *get_team_mut(&mut board, pulled_stone) = Team::Empty;
            *keep_checking = false;
        }
    }
}

fn new_board(rows: i32, columns: i32) -> Vec<Vec<Team>> {
    let mut board = Vec::new();
    for _ in 0..columns {
        let mut column = Vec::new();
        column.resize(rows as usize, Team::Empty);
        board.push(column);
    }
    board
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

fn draw_board_lines(rect: Rect, size_rows: i32, size_columns: i32) {
    let Rect { x, y, w, h } = rect;
    let dx = w / size_columns as f32;
    let dy = h / size_rows as f32;
    for vertical_line_i in 0..size_columns + 1 {
        let dx_i = vertical_line_i as f32 * dx;
        draw_line(x + dx_i, y, x + dx_i, y + h, 2.0, DARKGRAY);
    }

    for horizontal_line_i in 0..size_rows + 1 {
        let dy_i = horizontal_line_i as f32 * dy;
        draw_line(x, y + dy_i, x + w, y + dy_i, 2.0, DARKGRAY);
    }
}

fn get_tile(board_rect: Rect, (row_count, column_count): (i32, i32), pos: Vec2) -> Option<IVec2> {
    if board_rect.contains(pos) {
        let tile_size_x = board_rect.w / column_count as f32;
        let tile_size_y = board_rect.h / row_count as f32;
        let x_i = ((pos.x - board_rect.x) / tile_size_x) as i32;
        let y_i = ((pos.y - board_rect.y) / tile_size_y) as i32;
        Some(IVec2::new(x_i, y_i))
    } else {
        None
    }
}
fn get_team(board: &Vec<Vec<Team>>, tile: IVec2) -> Team {
    let row_count = board[0].len() as i32;
    let column_count = board.len() as i32;
    let x = ((tile.x + column_count) % column_count) as usize;
    let y = ((tile.y + row_count) % row_count) as usize;
    board[x][y]
}

fn get_team_mut(board: &mut Vec<Vec<Team>>, tile: IVec2) -> &mut Team {
    let row_count = board[0].len() as i32;
    let column_count = board.len() as i32;
    let x = ((tile.x + column_count) % column_count) as usize;
    let y = ((tile.y + row_count) % row_count) as usize;
    board.get_mut(x).unwrap().get_mut(y).unwrap()
}

fn draw_stones(board: &Vec<Vec<Team>>, board_rect: Rect, size_rows: i32, size_columns: i32) {
    for (x_i, column) in board.iter().enumerate() {
        for (y_i, team) in column.iter().enumerate() {
            let color = team.choose(TRANSPARENT, WHITE_FULL, BLACK_FULL);
            draw_stone(
                IVec2::new(x_i as i32, y_i as i32),
                color,
                board_rect,
                size_rows,
                size_columns,
            );
        }
    }
}
fn draw_stone(tile: IVec2, color: Color, board_rect: Rect, size_rows: i32, size_columns: i32) {
    let tile_size_x = board_rect.w / size_columns as f32;
    let tile_size_y = board_rect.h / size_rows as f32;
    draw_rectangle(
        board_rect.x + tile.x as f32 * tile_size_x,
        board_rect.y + tile.y as f32 * tile_size_y,
        tile_size_x,
        tile_size_y,
        color,
    );
}
fn draw_score(board_rect: Rect, board: &Vec<Vec<Team>>) {
    let mut whites = 0;
    let mut blacks = 0;
    for column in board {
        for team in column {
            match team {
                Team::Empty => {}
                Team::White => whites += 1,
                Team::Black => blacks += 1,
            }
        }
    }
    let font_size = score_font_size(screen_width(), screen_height());

    let white_str = format!("{}", whites);
    let white_dimensions = measure_text(&white_str, None, font_size as u16, 1.0);
    draw_text(
        &white_str,
        (board_rect.right() - white_dimensions.width).round(),
        (board_rect.y - 1.0 * white_dimensions.height).round(),
        font_size,
        WHITE,
    );
    let black_str = format!("{}", blacks);
    draw_text(
        &black_str,
        board_rect.left().round(),
        (board_rect.y - 1.0 * white_dimensions.height).round(),
        font_size,
        BLACK,
    );
}

fn score_font_size(screen_w: f32, screen_h: f32) -> f32 {
    choose_font_size(screen_w, screen_h) * 3.0
}

fn draw_instructions(buttons: &Buttons) {
    buttons.restart.render(&STYLE);
    buttons.undo.render(&STYLE);
}

fn draw_size(buttons: &Buttons) {
    buttons.rows.render(&STYLE);
    buttons.columns.render(&STYLE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tile_outside() {
        let tile = get_tile(
            Rect::new(20.0, 40.0, 100.0, 120.0),
            (8, 8),
            Vec2::new(0.0, 0.0),
        );
        assert_eq!(tile, None);
    }

    #[test]
    fn test_get_tile_inside() {
        let tile = get_tile(
            Rect::new(20.0, 40.0, 80.0, 80.0),
            (8, 8),
            Vec2::new(45.0, 55.0),
        );
        assert_eq!(tile, Some(IVec2::new(2, 1)));
    }

    #[test]
    fn test_put_stone_basic() {
        let mut board = new_board(5, 5);
        board[1][2] = Team::Black;
        let mut current_turn = Team::White;
        try_put_stone(&mut current_turn, &mut board, IVec2::new(3, 2));
        assert_eq!(current_turn, Team::Black);
        assert_eq!(board[1][2], Team::Empty);
        assert_eq!(board[2][2], Team::White);
        assert_eq!(board[3][2], Team::White);
    }

    #[test]
    fn test_put_stone_toroid() {
        let mut board = new_board(5, 5);
        board[1][2] = Team::Black;
        let mut current_turn = Team::White;
        try_put_stone(&mut current_turn, &mut board, IVec2::new(4, 2));
        assert_eq!(current_turn, Team::Black);
        assert_eq!(board[1][2], Team::Empty);
        assert_eq!(board[0][2], Team::White);
        assert_eq!(board[4][2], Team::White);
    }

    #[test]
    fn test_can_not_put_stone() {
        let mut board = new_board(5, 5);
        board[1][2] = Team::Black;
        let mut current_turn = Team::White;
        try_put_stone(&mut current_turn, &mut board, IVec2::new(1, 2));
        assert_eq!(current_turn, Team::White);
        assert_eq!(board[1][2], Team::Black);
    }
}
