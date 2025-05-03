use crate::scenes::game;
use crate::scenes::game::Buttons;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Team {
    Empty,
    White,
    Black,
}
impl Team {
    #[must_use]
    pub fn toggle(&self) -> Team {
        match self {
            Team::White => Team::Black,
            Team::Black => Team::White,
            _ => panic!("can not toggle a Team::Empty"),
        }
    }
    pub fn choose<T>(&self, if_empty: T, if_white: T, if_black: T) -> T {
        match self {
            Team::Empty => if_empty,
            Team::White => if_white,
            Team::Black => if_black,
        }
    }
}

pub type Board2d = Vec<Vec<Team>>;
type History = Vec<Board2d>;

pub struct Board {
    pub size_rows: i32,
    pub size_columns: i32,
    pub board: Board2d,
    pub board_history: History,
    pub turn: Team,
}

impl Board {
    pub fn new(size_rows: i32, size_columns: i32) -> Self {
        let board = new_board(size_rows, size_columns);
        let board_history = Vec::new();
        let turn = Team::White;
        Self {
            size_rows,
            size_columns,
            board,
            board_history,
            turn,
        }
    }
    pub fn new_default_size() -> Self {
        Self::new(7, 5)
    }
    pub fn reset(&mut self) {
        *self = Self::new(self.size_rows, self.size_columns);
    }

    pub fn size(&self) -> (i32, i32) {
        (self.size_rows, self.size_columns)
    }
    pub fn maybe_change_size(&mut self, buttons: &mut Buttons) {
        maybe_change_size(
            &mut self.size_rows,
            &mut self.size_columns,
            buttons,
            &mut self.board,
        )
    }
    pub fn pop_history(&mut self) {
        let previous = self.board_history.pop();
        if let Some(b) = previous {
            self.board = b;
            self.turn = self.turn.toggle()
        };
    }
    pub fn score(&self) -> (i32, i32) {
        compute_score(&self.board)
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

fn maybe_change_size(
    size_rows: &mut i32,
    size_columns: &mut i32,
    buttons: &mut game::Buttons, // TODO: extract to Actions to extract side-effects?
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

fn compute_score(board: &Vec<Vec<Team>>) -> (i32, i32) {
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
    (whites, blacks)
}
