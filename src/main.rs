use juquad::widgets::anchor::Anchor;
use juquad::widgets::button::{Button, Style};
use juquad::widgets::text::TextRect;
use macroquad::prelude::*;

const DEFAULT_WINDOW_WIDTH: i32 = 800;
const DEFAULT_WINDOW_HEIGHT: i32 = 800;
const DEFAULT_WINDOW_TITLE: &str = "orthomagnet";

const BOARD_TOP_COEF: f32 = 0.1;
const FONT_SIZE: f32 = 16.0;

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
    pub fn new(count: i32, position: Anchor, vertical_pad: f32) -> Self {
        let increase = Button::new("+", position, FONT_SIZE);
        let counter = TextRect::new(
            count.to_string().as_str(),
            from_below(increase.rect(), 0.0, vertical_pad),
            FONT_SIZE,
        );
        let decrease = Button::new("-", from_below(counter.rect, 0.0, vertical_pad), FONT_SIZE);
        let rect = increase
            .rect()
            .combine_with(counter.rect)
            .combine_with(decrease.rect());
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
            FONT_SIZE,
        )
    }
    pub fn render(&self, style: &Style) {
        self.increase.render(style);
        self.counter.render_text(style.text_color.at_rest);
        self.decrease.render(style);
    }
}
fn from_below(other: Rect, x_diff: f32, y_diff: f32) -> Anchor {
    Anchor::top_left(other.x + x_diff, other.y + other.h + y_diff)
}
pub struct Buttons {
    pub restart: Button,
    pub size_text: TextRect,
    pub rows: Counter,
    pub columns: Counter,
}

impl Buttons {
    pub fn new(screen_height: f32, row_count: i32, column_count: i32) -> Self {
        let left_pad = 16.0;
        let vert_pad = 10.0;
        let counter_inner_pad = 0.0;
        let restart = Button::new(
            "Restart (R)",
            Anchor::top_left(left_pad, (screen_height * BOARD_TOP_COEF).round()),
            FONT_SIZE,
        );
        let size_text = TextRect::new(
            "rows * columns:",
            from_below(restart.rect(), -left_pad, vert_pad * 4.0),
            FONT_SIZE,
        );
        let rows = Counter::new(
            row_count,
            Anchor::top_left(left_pad, size_text.rect.y + size_text.rect.h + vert_pad),
            counter_inner_pad,
        );
        let columns = Counter::new(
            column_count,
            Anchor::top_left(left_pad * 1.5 + rows.rect.w, rows.rect.y),
            counter_inner_pad,
        );
        Self {
            restart,
            size_text,
            rows,
            columns,
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut turn = Team::White;
    let mut size_rows = 5;
    let mut size_columns = 5;
    let mut board = new_board(size_rows, size_columns);
    let mut buttons = Buttons::new(screen_height(), size_rows, size_columns);
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::R) || buttons.restart.interact().is_clicked() {
            board = new_board(size_rows, size_columns);
        }
        maybe_change_size(&mut size_rows, &mut size_columns, &mut buttons, &mut board);
        clear_background(GRAY);
        let sw = screen_width();
        let sh = screen_height();

        let board_rect = Rect::new(sw * 0.2, sh * BOARD_TOP_COEF, sw * 0.6, sh * 0.6);
        draw_board_lines(board_rect, size_rows, size_columns);
        if let Some(tile) = get_tile(
            board_rect,
            (size_rows, size_columns),
            Vec2::from(mouse_position()),
        ) {
            let color = turn.choose(TRANSPARENT, WHITE_HINT, BLACK_HINT);
            draw_stone(tile, color, board_rect, size_rows, size_columns);
            if is_mouse_button_released(MouseButton::Left) {
                try_put_stone(&mut turn, &mut board, tile);
            }
        }
        draw_stones(&board, board_rect, size_rows, size_columns);
        draw_score(board_rect, &board);
        draw_instructions(&buttons);
        draw_size(&buttons);
        next_frame().await
    }
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
    let sw = screen_width();
    let font_size = FONT_SIZE * 3.0;

    let white_str = format!("{}", whites);
    let white_dimensions = measure_text(&white_str, None, font_size as u16, 1.0);
    draw_text(
        &white_str,
        (sw * 0.5 - white_dimensions.width * 0.5).round(),
        (board_rect.y - 1.0 * white_dimensions.height).round(),
        font_size,
        WHITE,
    );
    let black_str = format!("{}", blacks);
    let black_dimensions = measure_text(&black_str, None, font_size as u16, 1.0);
    draw_text(
        &black_str,
        (sw * 0.5 - black_dimensions.width * 0.5).round(),
        (board_rect.y + board_rect.h + black_dimensions.height + black_dimensions.offset_y).round(),
        font_size,
        BLACK,
    );
}

fn draw_instructions(buttons: &Buttons) {
    static STYLE: Style = Style::new();
    buttons.restart.render(&STYLE);
}

fn draw_size(buttons: &Buttons) {
    static STYLE: Style = Style::new();
    buttons.size_text.render_text(STYLE.text_color.at_rest);
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
