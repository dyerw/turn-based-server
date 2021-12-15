use futures::{SinkExt, StreamExt};
use multi_chess::{
    codec::MessageCodec,
    game::{Color, Position},
    messages::Message,
};
use std::future::ready;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite};

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:1337").await.unwrap();
    let (stream_read, stream_write) = stream.split();

    let source = FramedRead::new(stream_read, MessageCodec {});
    let mut sink = FramedWrite::new(stream_write, MessageCodec {});
    sink.feed(Message::CreateLobby {
        name: "new lobby".into(),
    })
    .await
    .unwrap();
    sink.feed(Message::ListLobbiesRequest).await.unwrap();
    sink.flush().await.unwrap();
    source
        .for_each(move |msg| {
            println!("Received {:?}", msg);
            ready(())
        })
        .await;
    // sink.feed(Frame::Game(GameMessage::MovePiece {
    //     player: Color::W,
    //     from: Position { x: 1, y: 1 },
    //     to: Position { x: 1, y: 2 },
    // }))
    // .await
    // .unwrap();
    // sink.feed(Frame::Game(GameMessage::MovePiece {
    //     player: Color::W,
    //     from: Position { x: 1, y: 2 },
    //     to: Position { x: 1, y: 3 },
    // }))
    // .await
    // .unwrap();
    // sink.feed(Frame::Game(GameMessage::MovePiece {
    //     player: Color::W,
    //     from: Position { x: 0, y: 0 },
    //     to: Position { x: 0, y: 4 },
    // }))
    // .await
    // .unwrap();
    // sink.feed(Frame::Game(GameMessage::MovePiece {
    //     player: Color::B,
    //     from: Position { x: 5, y: 5 },
    //     to: Position { x: 7, y: 7 },
    // }))
    // .await
    // .unwrap();
    sink.flush().await.unwrap();
    sink.close().await.unwrap();
    println!("Done");
}
