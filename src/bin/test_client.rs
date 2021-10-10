use futures::SinkExt;
use multi_chess::{
    frame::{Frame, FrameCodec, PlayerAction},
    game::{Color, Position},
};
use tokio::net::TcpStream;
use tokio_util::codec::FramedWrite;

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:1337").await.unwrap();
    let mut sink = FramedWrite::new(stream, FrameCodec {});
    sink.feed(Frame::PlayerAction(PlayerAction::MovePiece {
        player: Color::W,
        from: Position { x: 1, y: 1 },
        to: Position { x: 1, y: 2 },
    }))
    .await
    .unwrap();
    sink.feed(Frame::PlayerAction(PlayerAction::MovePiece {
        player: Color::W,
        from: Position { x: 1, y: 2 },
        to: Position { x: 1, y: 3 },
    }))
    .await
    .unwrap();
    sink.feed(Frame::PlayerAction(PlayerAction::MovePiece {
        player: Color::W,
        from: Position { x: 0, y: 0 },
        to: Position { x: 0, y: 4 },
    }))
    .await
    .unwrap();
    sink.feed(Frame::PlayerAction(PlayerAction::MovePiece {
        player: Color::B,
        from: Position { x: 5, y: 5 },
        to: Position { x: 7, y: 7 },
    }))
    .await
    .unwrap();
    sink.flush().await.unwrap();
    sink.close().await.unwrap();
    println!("Done");
}
