use futures::{SinkExt, StreamExt};
use multi_chess::{codec::MessageCodec, messages::NetworkMessage};
use std::future::ready;
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:1337").await.unwrap();
    let (stream_read, stream_write) = stream.split();

    let source = FramedRead::new(stream_read, MessageCodec {});
    let mut sink = FramedWrite::new(stream_write, MessageCodec {});

    sink.feed(NetworkMessage::ListLobbiesRequest).await.unwrap();
    sink.feed(NetworkMessage::JoinLobby {
        name: "new lobby2".into(),
    })
    .await
    .unwrap();
    sink.flush().await.unwrap();

    source
        .for_each(move |msg| {
            println!("Received {:?}", msg);
            ready(())
        })
        .await;
    sink.flush().await.unwrap();
    sink.close().await.unwrap();
    println!("Done");
}
