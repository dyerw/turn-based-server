use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::{
    frame::{Frame, FrameCodec, PlayerAction},
    server_state::ServerState,
};

pub struct Peer {
    socket: Framed<TcpStream, FrameCodec>,
    server_state: ServerState,
}

impl Peer {
    pub fn new(socket: Framed<TcpStream, FrameCodec>, server_state: ServerState) -> Peer {
        Peer {
            socket,
            server_state,
        }
    }

    pub async fn process(&mut self) {
        loop {
            let frame = self.socket.next().await;
            match frame {
                Some(r) => match r {
                    Ok(f) => self.process_frame(f),
                    Err(fe) => {
                        println!("Frame Error: {:?}", fe);
                    }
                },
                None => {}
            }
        }
    }

    fn process_frame(&mut self, frame: Frame) {
        match frame {
            Frame::PlayerAction(action) => {
                self.handle_player_action(action);
            }
            Frame::Lobby(action) => {}
        }
    }

    fn handle_player_action(&mut self, action: PlayerAction) {
        let mut game = self.server_state.game.lock().unwrap();
        match action {
            PlayerAction::MovePiece { player, from, to } => match game.move_piece(from, to) {
                Ok(_) => {
                    println!("{}", game);
                }
                Err(ge) => {
                    println!("Game Error {:?}", ge)
                }
            },
        }
    }
}
