#[derive(Copy, Clone, Debug)]
pub enum Color {
    B,
    W,
}

enum Piece {
    K(Color),
    Q(Color),
    B(Color),
    N(Color),
    R(Color),
    P(Color),
}

enum Space {
    Empty,
    Piece(Piece),
}

#[derive(Debug)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

pub struct Game {
    board: [[Space; 8]; 8],
    turn: Color,
}

fn in_bounds(p: &Position) -> bool {
    return p.x <= 7 && p.x >= 0 && p.y >= 0 && p.x <= 7;
}

fn king_row(c: Color) -> [Space; 8] {
    [
        Space::Piece(Piece::R(c)),
        Space::Piece(Piece::N(c)),
        Space::Piece(Piece::B(c)),
        Space::Piece(Piece::Q(c)),
        Space::Piece(Piece::K(c)),
        Space::Piece(Piece::B(c)),
        Space::Piece(Piece::N(c)),
        Space::Piece(Piece::R(c)),
    ]
}

fn pawn_row(c: Color) -> [Space; 8] {
    [
        Space::Piece(Piece::P(c)),
        Space::Piece(Piece::P(c)),
        Space::Piece(Piece::P(c)),
        Space::Piece(Piece::P(c)),
        Space::Piece(Piece::P(c)),
        Space::Piece(Piece::P(c)),
        Space::Piece(Piece::P(c)),
        Space::Piece(Piece::P(c)),
    ]
}

fn empty_row() -> [Space; 8] {
    [
        Space::Empty,
        Space::Empty,
        Space::Empty,
        Space::Empty,
        Space::Empty,
        Space::Empty,
        Space::Empty,
        Space::Empty,
    ]
}

pub enum GameError {
    InvalidMove,
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: [
                king_row(Color::B),
                pawn_row(Color::B),
                empty_row(),
                empty_row(),
                empty_row(),
                empty_row(),
                pawn_row(Color::W),
                king_row(Color::W),
            ],
            turn: Color::W,
        }
    }

    pub fn move_piece(&mut self, from: Position, to: Position) -> Result<(), GameError> {
        if !in_bounds(&from) || !in_bounds(&to) {
            return Err(GameError::InvalidMove);
        }
        let mut from_space = &self.board[from.x as usize][from.y as usize];
        let mut to_space = &self.board[to.x as usize][to.y as usize];
        match (from_space, to_space) {
            (Space::Empty, _) => Err(GameError::InvalidMove),
            (_, Space::Piece(_)) => Err(GameError::InvalidMove),
            (Space::Piece(_), Space::Empty) => {
                to_space = from_space;
                from_space = &Space::Empty;
                Ok(())
            }
        }
    }
}
