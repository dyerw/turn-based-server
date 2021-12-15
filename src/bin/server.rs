use actix::{io::FramedWrite, spawn, Actor, StreamHandler};
use log::{debug, error, info};
use multi_chess::{
    actor::{lobby_manager::LobbyManager, session::Session},
    codec::MessageCodec,
};
use tokio::{io::split, net::TcpListener};
use tokio_util::codec::FramedRead;

#[actix_rt::main]
async fn main() {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:1337").await.unwrap();

    info!("Listening on port 1337");

    let lobby_manager = LobbyManager::default();
    let lobby_manager_addr = lobby_manager.start();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        info!(
            "Accepting connection from {}",
            socket.local_addr().unwrap().ip()
        );
        let (socket_read, socket_write) = split(socket);
        let weak_lobby_manager_addr = lobby_manager_addr.downgrade();

        spawn(async move {
            Session::create(|ctx| {
                let framed_write = FramedWrite::new(socket_write, MessageCodec, ctx);
                Session::add_stream(FramedRead::new(socket_read, MessageCodec), ctx);
                Session::new(weak_lobby_manager_addr, framed_write)
            });
        })
        .await
        .unwrap();
    }
}
