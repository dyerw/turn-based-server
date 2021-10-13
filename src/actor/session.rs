use crate::codec::{Frame, FrameCodec};
use crate::message::lobby::LobbyMessage;
use actix::fut::ready;
use actix::{
    io::FramedWrite, Actor, ActorFutureExt, Addr, Context, ContextFutureSpawner, WrapFuture,
};
use actix::{AsyncContext, StreamHandler};
use std::io;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;

use super::lobby::Lobby;
use super::lobby_manager::{
    LobbyManager, LobbyManagerMessage, LobbyManagerResponse, LobbyManagerResponseSuccess,
};

pub struct Session {
    id: usize,
    lobby: Option<Addr<Lobby>>,
    lobby_manager: Addr<LobbyManager>,
    tcp_stream_write: FramedWrite<Frame, WriteHalf<TcpStream>, FrameCodec>,
}

impl Actor for Session {
    type Context = Context<Self>;
}

impl StreamHandler<Result<Frame, io::Error>> for Session {
    fn handle(&mut self, item: Result<Frame, io::Error>, ctx: &mut Self::Context) {
        match item {
            Ok(Frame::Lobby(LobbyMessage::CreateLobby { name })) => {
                self.lobby_manager
                    .send(LobbyManagerMessage::CreateLobby {
                        name,
                        creating_session: ctx.address(),
                    })
                    .into_actor(self)
                    .then(|res, act, ctx| {
                        match res {
                            Ok(LobbyManagerResponse(Ok(
                                LobbyManagerResponseSuccess::CreatedLobby { addr },
                            ))) => {
                                act.lobby = Some(addr);
                            }
                            _ => println!("Oh no!"),
                        }
                        ready(())
                    })
                    .wait(ctx);
            }
            _ => {}
        }
    }
}
