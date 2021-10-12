use bytes::{Buf, BytesMut};

use crate::game::{Color, Position};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum GameMessage {
    MovePiece {
        player: Color,
        from: Position,
        to: Position,
    },
}

impl GameMessage {
    pub fn decode(src: &mut BytesMut) -> Result<Option<Self>, GameMessageError> {
        match src[0] {
            0x01 => {
                if src.len() < 4 {
                    return Ok(None);
                }
                let color_byte = src[1];
                let from_byte = src[2];
                let to_byte = src[3];

                src.advance(4);
                match color_byte {
                    0x01 => Ok(Some(make_move_frame(Color::W, from_byte, to_byte))),
                    0x02 => Ok(Some(make_move_frame(Color::B, from_byte, to_byte))),
                    b => Err(GameMessageError::InvalidColorByte(b)),
                }
            }
            _ => Err(GameMessageError::CannotDecodeNonGameMessage),
        }
    }

    pub fn encode(item: Self, dst: &mut BytesMut) -> Result<(), GameMessageError> {
        match item {
            GameMessage::MovePiece { player, from, to } => {
                let color_byte: u8 = match player {
                    Color::W => 0x01u8,
                    Color::B => 0x02u8,
                };
                let frame: &[u8] = &[
                    0x01u8,
                    color_byte,
                    position_to_byte(&from),
                    position_to_byte(&to),
                ];
                dst.extend_from_slice(frame);
                Ok(())
            }
        }
    }
}

fn position_to_byte(p: &Position) -> u8 {
    return (p.x << 4) | p.y;
}

fn make_move_frame(color: Color, from: u8, to: u8) -> GameMessage {
    let from_x: u8 = from >> 4;
    let from_y: u8 = from & 0x0Fu8;
    let to_x: u8 = to >> 4;
    let to_y: u8 = to & 0x0Fu8;
    GameMessage::MovePiece {
        player: color,
        from: Position {
            x: from_x,
            y: from_y,
        },
        to: Position { x: to_x, y: to_y },
    }
}

#[derive(Debug, PartialEq)]
pub enum GameMessageError {
    InvalidColorByte(u8),
    CannotDecodeNonGameMessage,
}

#[cfg(test)]
mod tests {
    use super::position_to_byte;
    use crate::game::Position;

    #[test]
    fn test_position_to_byte() {
        let p1 = Position { x: 1, y: 1 };
        let p2 = Position { x: 8, y: 5 };
        assert_eq!(position_to_byte(&p1), 0x11u8);
        assert_eq!(position_to_byte(&p2), 0x85u8);
    }
}
