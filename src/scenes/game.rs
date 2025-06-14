use crate::board::{Board, Team};
use crate::remote_player::Command;
use crate::scenes::loading::Textures;
use crate::scenes::menu::Player;
use crate::ui::button_trait::ButtonTrait;
use crate::ui::complex_button::ComplexButton;
use crate::ui::counter::Counter;
use crate::{choose_font_size, new_button, AnyError, BASE_FONT_SIZE, STYLE};
use juquad::widgets::anchor::Anchor;
use juquad::widgets::button::Button;
use juquad::widgets::text::TextRect;
use juquad::widgets::Widget;
use macroquad::color::{Color, BLACK, DARKGRAY, WHITE};
use macroquad::input::{is_mouse_button_released, mouse_position, MouseButton};
use macroquad::math::{IVec2, Rect, Vec2};
use macroquad::prelude::{
    clear_background, draw_line, draw_rectangle, draw_text, is_key_down, is_key_pressed,
    is_mouse_button_pressed, measure_text, next_frame, screen_height, screen_width, KeyCode,
    Texture2D, GRAY,
};
use std::sync::mpsc::{Receiver, Sender};

const BOARD_TOP_COEF: f32 = 0.12;
const BOARD_LEFT_COEF: f32 = 0.15;
const BOARD_WIDTH_COEF: f32 = 1.0 - 2.0 * BOARD_LEFT_COEF;
const BOARD_HEIGHT_COEF: f32 = 0.58;

const WHITE_HINT: Color = Color::new(1.0, 1.0, 1.0, 0.3);
const BLACK_HINT: Color = Color::new(0.0, 0.0, 0.0, 0.3);
const WHITE_FULL: Color = Color::new(1.0, 1.0, 1.0, 0.7);
const BLACK_FULL: Color = Color::new(0.0, 0.0, 0.0, 0.7);
const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);

pub async fn scene(
    textures: Textures,
    player: Player,
    from_remote: Option<Receiver<Command>>,
    to_remote: Option<Sender<Command>>,
) -> Result<(), AnyError> {
    let mut width = screen_width();
    let mut height = screen_height();
    let mut board = Board::new_default_size();
    let (mut _font_size, mut buttons) = reset(
        width,
        height,
        board.size_rows,
        board.size_columns,
        &textures,
    );
    let (remote_color, local_color, local_team) = if let Player::Client = player {
        (WHITE_HINT, BLACK_HINT, Team::Black)
    } else {
        // TODO: any way to avoid setting this on Player::Local?
        (BLACK_HINT, WHITE_HINT, Team::White)
    };
    let mut remote_mouse = None;
    let mut previous_mouse_tile = None;
    clear_background(GRAY);
    next_frame().await; // ignore last click
    loop {
        let new_width = screen_width();
        let new_height = screen_height();
        if new_width != width || new_height != height {
            width = new_width;
            height = new_height;
            (_font_size, buttons) = reset(
                width,
                height,
                board.size_rows,
                board.size_columns,
                &textures,
            );
        }
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::R) || buttons.restart.interact().is_clicked() {
            board.reset();
        }
        if is_key_pressed(KeyCode::Z) && is_key_down(KeyCode::LeftControl)
            || buttons.undo.interact().is_clicked()
        {
            board.pop_history();
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            println!("{}", Vec2::from(mouse_position()));
        }
        board.maybe_change_size(&mut buttons);
        if buttons.toggle_shadows.interact().is_clicked() {
            // unsafe {
            //     SHADOWS = !SHADOWS;
            // }
        }
        clear_background(GRAY);

        let board_rect = Rect::new(
            width * BOARD_LEFT_COEF,
            height * BOARD_TOP_COEF,
            width * BOARD_WIDTH_COEF,
            height * BOARD_HEIGHT_COEF,
        );
        draw_board_lines(board_rect, board.size_rows, board.size_columns);
        match player {
            Player::Local => {
                let local_color = board.turn.choose(TRANSPARENT, WHITE_HINT, BLACK_HINT);
                let mouse_pos = Vec2::from(mouse_position());
                let new_tile = get_tile(board_rect, board.size(), mouse_pos);
                if let Some(tile) = new_tile {
                    draw_stone(tile, local_color, board_rect, board.size());
                    maybe_put_stone(&mut board, tile);
                }
            }
            Player::Server | Player::Client => {
                update_mouses(
                    &mut board,
                    &mut remote_mouse,
                    &mut previous_mouse_tile,
                    board_rect,
                    remote_color,
                    local_color,
                    local_team,
                    to_remote.as_ref().unwrap(),
                    from_remote.as_ref().unwrap(),
                )?;
            }
        }
        draw_stones(&board.board, board_rect, board.size());
        draw_score(board_rect, &board);
        draw_instructions(&buttons);
        draw_size(&buttons);
        next_frame().await
    }
    Ok(())
}

fn maybe_put_stone(board: &mut Board, tile: IVec2) {
    if is_mouse_button_released(MouseButton::Left) {
        board.board_history.push(board.board.clone());
        let previous_turn = board.turn;
        try_put_stone(&mut board.turn, &mut board.board, tile);
        if board.turn == previous_turn {
            board.board_history.pop();
        }
    }
}

fn reset(
    width: f32,
    height: f32,
    row_count: i32,
    column_count: i32,
    textures: &Textures,
) -> (f32, Buttons) {
    let font_size = choose_font_size(width, height) * 2.0;
    let buttons = Buttons::new(width, height, row_count, column_count, textures);
    (font_size, buttons)
}

pub struct Buttons {
    pub restart: ComplexButton,
    pub undo: ComplexButton,
    pub rows: Counter,
    pub columns: Counter,
    pub toggle_shadows: Button,
}

impl Buttons {
    pub fn new(
        screen_width: f32,
        screen_height: f32,
        row_count: i32,
        column_count: i32,
        textures: &Textures,
    ) -> Self {
        let mut font_size = choose_font_size(screen_width, screen_height);
        let texture_size_coef = font_size / BASE_FONT_SIZE;
        let font_size_coef = 1.2;
        font_size *= font_size_coef;
        let left_pad = 16.0;
        let counter_inner_pad = 0.0;
        let left = (BOARD_LEFT_COEF * screen_width).round();
        let bottom = (screen_height * (1.0 - BOARD_TOP_COEF)
            + score_font_size(screen_width, screen_height))
        .round();

        let texture_size = Vec2::new(textures.restart.width(), textures.restart.height())
            * 2.0
            * texture_size_coef;
        let undo_anchor = Anchor::bottom_left(left, bottom);
        let new_complex_button = |anchor, texture: Texture2D, text| {
            ComplexButton::new(anchor, vec![texture], texture_size, text, font_size)
        };
        let undo = new_complex_button(undo_anchor, textures.undo, "Undo");

        let restart_anchor = Anchor::bottom_left(undo.rect().x, undo.rect().y - undo.rect().h);
        let restart = new_complex_button(restart_anchor, textures.restart, "Restart");

        let anchor_columns =
            Anchor::bottom_right(((1.0 - BOARD_LEFT_COEF) * screen_width).round(), bottom);
        let columns = Counter::new(column_count, anchor_columns, counter_inner_pad, font_size);
        let anchor_rows = Anchor::top_right(columns.rect().x - left_pad * 0.5, columns.rect().y);
        let rows = Counter::new(row_count, anchor_rows, counter_inner_pad, font_size);

        let anchor = Anchor::top_left(0.0, 0.0);
        let toggle_shadows = new_button("toggle shadows", anchor, font_size);
        Self {
            restart,
            undo,
            rows,
            columns,
            toggle_shadows,
        }
    }
}
fn update_mouses(
    board: &mut Board,
    remote_mouse: &mut Option<IVec2>,
    previous_mouse_tile: &mut Option<IVec2>,
    board_rect: Rect,
    remote_color: Color,
    local_color: Color,
    local_team: Team,
    to_remote: &Sender<Command>,
    from_remote: &Receiver<Command>,
) -> Result<(), AnyError> {
    update_remote_mouse(remote_mouse, from_remote);
    if let Some(tile) = remote_mouse.as_ref() {
        draw_stone(*tile, remote_color, board_rect, board.size());
    }
    let new_tile_opt = update_local_mouse(board, local_team, board_rect, local_color);
    send_local_mouse_update(previous_mouse_tile, to_remote, new_tile_opt)?;
    Ok(())
}

fn update_remote_mouse(remote_mouse: &mut Option<IVec2>, from_remote: &Receiver<Command>) {
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
    board: &mut Board,
    local_team: Team,
    board_rect: Rect,
    local_color: Color,
) -> Option<IVec2> {
    let mouse_pos = Vec2::from(mouse_position());
    let new_tile = get_tile(board_rect, board.size(), mouse_pos);
    if let Some(tile) = new_tile {
        draw_stone(tile, local_color, board_rect, board.size());
        if board.turn == local_team {
            maybe_put_stone(board, tile);
        }
    }
    new_tile
}

fn send_local_mouse_update(
    previous_mouse_tile: &mut Option<IVec2>,
    to_remote: &Sender<Command>,
    new_tile_opt: Option<IVec2>,
) -> Result<(), AnyError> {
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

fn draw_stones(board: &Vec<Vec<Team>>, board_rect: Rect, board_size: (i32, i32)) {
    for (x_i, column) in board.iter().enumerate() {
        for (y_i, team) in column.iter().enumerate() {
            let color = team.choose(TRANSPARENT, WHITE_FULL, BLACK_FULL);
            draw_stone(
                IVec2::new(x_i as i32, y_i as i32),
                color,
                board_rect,
                board_size,
            );
        }
    }
}
fn draw_stone(tile: IVec2, color: Color, board_rect: Rect, (size_rows, size_columns): (i32, i32)) {
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
fn draw_score(board_rect: Rect, board: &Board) {
    let (whites, blacks) = board.score();
    let white_str = format!("{}", whites);
    let font_size = score_font_size(screen_width(), screen_height());
    let white_dimensions = measure_text(&white_str, None, font_size as u16, 1.0);
    let height = (board_rect.y - 1.0 * white_dimensions.height).round();
    draw_text(
        &white_str,
        (board_rect.right() - white_dimensions.width).round(),
        height,
        font_size,
        WHITE,
    );
    let black_str = format!("{}", blacks);
    draw_text(
        &black_str,
        board_rect.left().round(),
        height,
        font_size,
        BLACK,
    );
}

fn score_font_size(screen_w: f32, screen_h: f32) -> f32 {
    choose_font_size(screen_w, screen_h) * 3.0
}

fn draw_instructions(buttons: &Buttons) {
    buttons.restart.render(&STYLE);
    // draw_rect_lines(text_border(&buttons.restart.text_rect), 2.0, macroquad::prelude::RED);
    buttons.undo.render(&STYLE);
    // draw_rect_lines(text_border(&buttons.undo.text_rect), 2.0, macroquad::prelude::RED);
    // buttons.toggle_shadows.render(&STYLE);
}

#[allow(unused)]
fn text_border(rect: &TextRect) -> Rect {
    Rect::new(
        (rect.rect.x + rect.pad.x).round(),
        (rect.rect.y + rect.pad.y).round(),
        rect.text_width.round(),
        rect.text_height.round(),
    )
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
        let mut _board = Board::new(5, 5);
        let mut board = _board.board;
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
        let mut _board = Board::new(5, 5);
        let mut board = _board.board;
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
        let mut _board = Board::new(5, 5);
        let mut board = _board.board;
        board[1][2] = Team::Black;
        let mut current_turn = Team::White;
        try_put_stone(&mut current_turn, &mut board, IVec2::new(1, 2));
        assert_eq!(current_turn, Team::White);
        assert_eq!(board[1][2], Team::Black);
    }
}
