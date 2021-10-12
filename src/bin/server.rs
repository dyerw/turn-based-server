use multi_chess::{codec::FrameCodec, peer::Peer, server_state::ServerState};
use tokio::net::TcpListener;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:1337").await.unwrap();

    println!("Listening on port 1337");

    let server_state: ServerState = ServerState::new();

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let server_state = server_state.clone();
        let framed_socket = Framed::new(socket, FrameCodec {});
        let mut peer = Peer::new(framed_socket, server_state);
        tokio::spawn(async move {
            peer.process().await;
        });
    }
}
