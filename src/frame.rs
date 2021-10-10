use bytes::{Buf, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

use crate::game::{Color, Position};

#[derive(Debug)]
pub enum PlayerAction {
    MovePiece {
        player: Color,
        from: Position,
        to: Position,
    },
}

#[derive(Debug)]
pub enum PlayerActionError {
    InvalidColorByte(u8),
}

#[derive(Debug)]
pub enum FrameError {
    IoError(io::Error),
    InvalidFrameType(u8),
    InvalidFrameData(PlayerActionError),
    Incomplete,
}

impl From<io::Error> for FrameError {
    fn from(error: io::Error) -> Self {
        FrameError::IoError(error)
    }
}

#[derive(Debug)]
pub enum Frame {
    PlayerAction(PlayerAction),
}

pub struct FrameCodec {}

fn make_move_frame(color: Color, from: u8, to: u8) -> Frame {
    let from_x: u8 = from >> 4;
    let from_y: u8 = from & 0x0Fu8;
    let to_x: u8 = to >> 4;
    let to_y: u8 = to & 0x0Fu8;
    Frame::PlayerAction(PlayerAction::MovePiece {
        player: color,
        from: Position {
            x: from_x,
            y: from_y,
        },
        to: Position { x: to_x, y: to_y },
    })
}

impl Decoder for FrameCodec {
    type Item = Frame;
    type Error = FrameError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // No bytes to read
        if src.len() == 0 {
            return Ok(None);
        }

        match src[0] {
            // MovePiece Frame
            // len: 1 byte header, 1 byte color, 1 byte from, 1 byte to
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
                    b => Err(FrameError::InvalidFrameData(
                        PlayerActionError::InvalidColorByte(b),
                    )),
                }
            }
            b => {
                src.advance(1);
                Err(FrameError::InvalidFrameType(b))
            }
        }
    }
}

fn position_to_byte(p: &Position) -> u8 {
    return (p.x << 4) | p.y;
}

impl Encoder<Frame> for FrameCodec {
    type Error = FrameError;
    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            Frame::PlayerAction(a) => match a {
                PlayerAction::MovePiece { player, from, to } => {
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
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::position_to_byte;
    use crate::game::Position;

    #[test]
    fn test_position_to_byte() {
        let p1 = Position { x: 1, y: 1 };
        let p2 = Position { x: 8, y: 5 };
        assert_eq!(position_to_byte(&p1), 0x11u8);
        assert_eq!(position_to_byte(&p2), 0x85u8);
    }
}
