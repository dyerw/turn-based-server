use crate::codec::{Frame, FrameCodec, FrameError};
use crate::message::lobby::LobbyMessage;
use actix::fut::ready;
use actix::io::WriteHandler;
use actix::{
    io::FramedWrite, Actor, ActorFutureExt, Addr, Context, ContextFutureSpawner, WrapFuture,
};
use actix::{AsyncContext, StreamHandler, WeakAddr};
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::WriteHalf;
use tokio::net::TcpStream;

fn get_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

use super::lobby::Lobby;
use super::lobby_manager::{
    LobbyManager, LobbyManagerMessage, LobbyManagerResponse, LobbyManagerResponseSuccess,
};

pub struct Session {
    id: usize,
    lobby: Option<Addr<Lobby>>,
    lobby_manager: WeakAddr<LobbyManager>,
    tcp_stream_write: FramedWrite<Frame, WriteHalf<TcpStream>, FrameCodec>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session").field("id", &self.id).finish()
    }
}

impl Session {
    pub fn new(
        lobby_manager: WeakAddr<LobbyManager>,
        tcp_stream_write: FramedWrite<Frame, WriteHalf<TcpStream>, FrameCodec>,
    ) -> Session {
        println!("Created Session");
        Session {
            id: get_id(),
            lobby: None,
            lobby_manager,
            tcp_stream_write,
        }
    }
}

impl WriteHandler<FrameError> for Session {}

impl Actor for Session {
    type Context = Context<Self>;
}

impl StreamHandler<Result<Frame, FrameError>> for Session {
    fn handle(&mut self, item: Result<Frame, FrameError>, ctx: &mut Self::Context) {
        match item {
            Ok(Frame::Lobby(LobbyMessage::CreateLobby { name })) => {
                let lm_addr = self.lobby_manager.upgrade();

                match lm_addr {
                    Some(a) => {
                        a.send(LobbyManagerMessage::CreateLobby {
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
                    None => {
                        println!("Lobby Manager Addr not available!");
                    }
                }
            }
            _ => {}
        }
    }
}
