use std::fmt;

#[derive(Copy, Clone, Debug)]
pub enum Color {
    B,
    W,
}

#[derive(Copy, Clone, Debug)]
pub enum Piece {
    K(Color),
    Q(Color),
    B(Color),
    N(Color),
    R(Color),
    P(Color),
}

#[derive(Copy, Clone, Debug)]
pub enum Space {
    Empty,
    Piece(Piece),
}

fn space_to_unicode(s: &Space) -> String {
    match s {
        Space::Empty => String::from("■"),
        Space::Piece(p) => match p {
            Piece::K(c) => match c {
                Color::W => String::from("♔"),
                Color::B => String::from("♚"),
            },
            Piece::Q(c) => match c {
                Color::W => String::from("♕"),
                Color::B => String::from("♛"),
            },
            Piece::R(c) => match c {
                Color::W => String::from("♖"),
                Color::B => String::from("♜"),
            },
            Piece::B(c) => match c {
                Color::W => String::from("♗"),
                Color::B => String::from("♝"),
            },
            Piece::N(c) => match c {
                Color::W => String::from("♘"),
                Color::B => String::from("♞"),
            },
            Piece::P(c) => match c {
                Color::W => String::from("♙"),
                Color::B => String::from("♟︎"),
            },
        },
    }
}
#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug)]
pub struct Game {
    board: [[Space; 8]; 8],
    turn: Color,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = Ok(());
        for x in 0..8 {
            for y in 0..8 {
                result = result.and(write!(f, "{}", space_to_unicode(&self.board[x][y])));
            }
            result = result.and(write!(f, "\n"));
        }
        result
    }
}

fn in_bounds(p: &Position) -> bool {
    return p.x <= 7 && p.x <= 7;
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

#[derive(Debug)]
pub enum GameError {
    OutOfBoundsMove,
    InvalidMove {
        from: (Position, Space),
        to: (Position, Space),
    },
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
            return Err(GameError::OutOfBoundsMove);
        }
        let from_space = &self.board[from.y as usize][from.x as usize];
        let to_space = &self.board[to.y as usize][to.x as usize];
        let error = GameError::InvalidMove {
            from: (from.clone(), *from_space),
            to: (to.clone(), *to_space),
        };
        match (from_space, to_space) {
            (Space::Empty, _) => Err(error),
            (_, Space::Piece(_)) => Err(error),
            (Space::Piece(_), Space::Empty) => {
                self.board[to.y as usize][to.x as usize] = from_space.clone();
                self.board[from.y as usize][from.x as usize] = Space::Empty;
                Ok(())
            }
        }
    }
}
