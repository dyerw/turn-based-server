use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::{
    codec::{Frame, FrameCodec},
    game::Color,
    message::{game::GameMessage, lobby::LobbyMessage},
    server_state::ServerState,
};

enum LobbyState {
    OutOfLobby,
    InGame { name: String, color: Color },
}

pub struct Peer {
    socket: Framed<TcpStream, FrameCodec>,
    server_state: ServerState,
    lobby_state: LobbyState,
}

impl Peer {
    pub fn new(socket: Framed<TcpStream, FrameCodec>, server_state: ServerState) -> Peer {
        Peer {
            socket,
            server_state,
            lobby_state: LobbyState::OutOfLobby,
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
        println!("Processing frame {:?}", frame);
        match frame {
            Frame::Game(message) => {
                self.handle_game_message(message);
            }
            Frame::Lobby(message) => {
                self.handle_lobby_message(message);
            }
        }
    }

    fn handle_lobby_message(&mut self, message: LobbyMessage) {
        match message {
            LobbyMessage::CreateLobby { name } => {
                self.lobby_state = LobbyState::InGame {
                    name,
                    color: Color::W,
                };
            }
        }
    }

    fn handle_game_message(&mut self, message: GameMessage) {
        match &self.lobby_state {
            LobbyState::OutOfLobby => {}
            LobbyState::InGame { name, color } => {
                let mut game = self.server_state.game.lock().unwrap();
                match message {
                    GameMessage::MovePiece { player, from, to } => {
                        // if *color == player {
                        //     println!("Cannot move other players pieces!");
                        //     println!("Lobby state {:?} Move color {:?}", color, player);
                        //     return;
                        // }
                        match game.move_piece(from, to) {
                            Ok(_) => {
                                println!("{}", game);
                            }
                            Err(ge) => {
                                println!("Game Error {:?}", ge)
                            }
                        }
                    }
                }
            }
        }
    }
}
