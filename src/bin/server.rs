use std::sync::{Arc, Mutex};

use futures::StreamExt;
use multi_chess::{
    frame::{Frame, FrameCodec, PlayerAction},
    game::{Game, GameError},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

type ServerState = Arc<Mutex<Game>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:1337").await.unwrap();

    println!("Listening on port 1337");

    let server_state: ServerState = Arc::new(Mutex::new(Game::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let server_state = server_state.clone();
        tokio::spawn(async move {
            process(socket, server_state).await;
        });
    }
}

async fn process(stream: TcpStream, server_state: ServerState) {
    let mut frame_stream = Framed::new(stream, FrameCodec {});
    loop {
        let frame = frame_stream.next().await;
        match frame {
            Some(r) => match r {
                Ok(f) => {
                    println!("{:#?}", f);
                    match f {
                        Frame::PlayerAction(action) => {
                            let mut game = server_state.lock().unwrap();
                            match handle_player_action(action, &mut game) {
                                Ok(_) => {}
                                Err(ge) => {
                                    println!("Game error {:?}", ge);
                                }
                            }
                            println!("{}", game);
                        }
                    }
                }
                Err(e) => {}
            },
            _ => {}
        }
    }
}

fn handle_player_action(a: PlayerAction, game: &mut Game) -> Result<(), GameError> {
    match a {
        PlayerAction::MovePiece { player, from, to } => game.move_piece(from, to),
    }
}
