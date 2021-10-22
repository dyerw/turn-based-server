use bytes::{Buf, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

use crate::message::{
    game::{GameMessage, GameMessageError},
    lobby::{LobbyMessage, LobbyMessageError},
};

#[derive(Debug)]
pub enum FrameError {
    IoError(io::Error),
    InvalidFrameType(u8),
    InvalidGameMessage(GameMessageError),
    InvalidLobbyMessage(LobbyMessageError),
    Incomplete,
}

impl From<io::Error> for FrameError {
    fn from(error: io::Error) -> Self {
        FrameError::IoError(error)
    }
}

#[derive(Debug, PartialEq)]
pub enum Frame {
    Game(GameMessage),
    Lobby(LobbyMessage),
}

pub struct FrameCodec;

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
            0x01 => GameMessage::decode(src)
                .map(|o| o.map(Frame::Game))
                .map_err(FrameError::InvalidGameMessage),
            0x10 => LobbyMessage::decode(src)
                .map(|o| o.map(Frame::Lobby))
                .map_err(FrameError::InvalidLobbyMessage),
            b => {
                src.advance(1);
                Err(FrameError::InvalidFrameType(b))
            }
        }
    }
}

impl Encoder<Frame> for FrameCodec {
    type Error = FrameError;
    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            Frame::Game(i) => GameMessage::encode(i, dst).map_err(FrameError::InvalidGameMessage),
            Frame::Lobby(i) => {
                LobbyMessage::encode(i, dst).map_err(FrameError::InvalidLobbyMessage)
            }
        }
    }
}
