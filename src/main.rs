use macroquad::prelude::*;

const SIZE: i32 = 5;

const DEFAULT_WINDOW_WIDTH: i32 = 800;
const DEFAULT_WINDOW_HEIGHT: i32 = 800;
const DEFAULT_WINDOW_TITLE: &str = "orthomagnet";

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

#[macroquad::main(window_conf)]
async fn main() {
    let mut turn = Team::White;
    let mut board = new_board(SIZE);
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::R) {
            board = new_board(SIZE);
        }
        clear_background(GRAY);
        let sw = screen_width();

        let board_rect = Rect::new(sw * 0.2, sw * 0.1, sw * 0.6, sw * 0.6);
        draw_board_lines(board_rect);
        if let Some(tile) = get_tile(board_rect, SIZE, Vec2::from(mouse_position())) {
            let color = turn.choose(TRANSPARENT, WHITE_HINT, BLACK_HINT);
            draw_stone(tile, color, board_rect);
            if is_mouse_button_released(MouseButton::Left) {
                try_put_stone(&mut turn, &mut board, tile);
            }
        }
        draw_stones(&board, board_rect);
        next_frame().await
    }
}

fn try_put_stone(turn: &mut Team, mut board: &mut Vec<Vec<Team>>, tile: IVec2) {
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
        for i in 2..SIZE {
            check_direction(
                turn,
                opponent,
                tile,
                side_xp,
                IVec2::new(i, 0),
                &mut board,
                &mut check_xp,
            );
            if check_xn {
                let pulled = tile + IVec2::new(-i, 0);
                if *turn == get_team(&board, pulled) {
                    check_xn = false;
                }
                if opponent == get_team(&board, pulled) {
                    *get_team_mut(&mut board, side_xn) = *turn;
                    *get_team_mut(&mut board, pulled) = Team::Empty;
                    check_xn = false;
                }
            }
            if check_yp {
                let pulled = tile + IVec2::new(0, i);
                if *turn == get_team(&board, pulled) {
                    check_yp = false;
                }
                if opponent == get_team(&board, pulled) {
                    *get_team_mut(&mut board, side_yp) = *turn;
                    *get_team_mut(&mut board, pulled) = Team::Empty;
                    check_yp = false;
                }
            }
            if check_yn {
                let pulled = tile + IVec2::new(0, -i);
                if *turn == get_team(&board, pulled) {
                    check_yn = false;
                }
                if opponent == get_team(&board, pulled) {
                    *get_team_mut(&mut board, side_yn) = *turn;
                    *get_team_mut(&mut board, pulled) = Team::Empty;
                    check_yn = false;
                }
            }
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

fn new_board(size: i32) -> Vec<Vec<Team>> {
    let mut board = Vec::new();
    for _ in 0..size {
        let mut column = Vec::new();
        column.resize(size as usize, Team::Empty);
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

fn draw_board_lines(rect: Rect) {
    let Rect { x, y, w, h } = rect;
    let dx = w / SIZE as f32;
    let dy = h / SIZE as f32;
    for vertical_line_i in 0..SIZE + 1 {
        let dx_i = vertical_line_i as f32 * dx;
        draw_line(x + dx_i, y, x + dx_i, y + h, 2.0, DARKGRAY);
    }

    for horizontal_line_i in 0..SIZE + 1 {
        let dy_i = horizontal_line_i as f32 * dy;
        draw_line(x, y + dy_i, x + w, y + dy_i, 2.0, DARKGRAY);
    }
}

fn get_tile(board_rect: Rect, size: i32, pos: Vec2) -> Option<IVec2> {
    if board_rect.contains(pos) {
        let tile_size_x = board_rect.w / size as f32;
        let tile_size_y = board_rect.h / size as f32;
        let x_i = ((pos.x - board_rect.x) / tile_size_x) as i32;
        let y_i = ((pos.y - board_rect.y) / tile_size_y) as i32;
        Some(IVec2::new(x_i, y_i))
    } else {
        None
    }
}
fn get_team(board: &Vec<Vec<Team>>, tile: IVec2) -> Team {
    let x = ((tile.x + SIZE) % SIZE) as usize;
    let y = ((tile.y + SIZE) % SIZE) as usize;
    board[x][y]
}

fn get_team_mut(board: &mut Vec<Vec<Team>>, tile: IVec2) -> &mut Team {
    let x = ((tile.x + SIZE) % SIZE) as usize;
    let y = ((tile.y + SIZE) % SIZE) as usize;
    board.get_mut(x).unwrap().get_mut(y).unwrap()
}

fn draw_stones(board: &Vec<Vec<Team>>, board_rect: Rect) {
    for (x_i, column) in board.iter().enumerate() {
        for (y_i, team) in column.iter().enumerate() {
            let color = team.choose(TRANSPARENT, WHITE_FULL, BLACK_FULL);
            draw_stone(IVec2::new(x_i as i32, y_i as i32), color, board_rect);
        }
    }
}
fn draw_stone(tile: IVec2, color: Color, board_rect: Rect) {
    let tile_size_x = board_rect.w / SIZE as f32;
    let tile_size_y = board_rect.h / SIZE as f32;
    draw_rectangle(
        board_rect.x + tile.x as f32 * tile_size_x,
        board_rect.y + tile.y as f32 * tile_size_y,
        tile_size_x,
        tile_size_y,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tile_outside() {
        let tile = get_tile(Rect::new(20.0, 40.0, 100.0, 120.0), 8, Vec2::new(0.0, 0.0));
        assert_eq!(tile, None);
    }

    #[test]
    fn test_get_tile_inside() {
        let tile = get_tile(Rect::new(20.0, 40.0, 80.0, 80.0), 8, Vec2::new(45.0, 55.0));
        assert_eq!(tile, Some(IVec2::new(2, 1)));
    }

    #[test]
    fn test_put_stone_basic() {
        let mut board = new_board(SIZE);
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
        let mut board = new_board(SIZE);
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
        let mut board = new_board(SIZE);
        board[1][2] = Team::Black;
        let mut current_turn = Team::White;
        try_put_stone(&mut current_turn, &mut board, IVec2::new(1, 2));
        assert_eq!(current_turn, Team::White);
        assert_eq!(board[1][2], Team::Black);
    }
}
