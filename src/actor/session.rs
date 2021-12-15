use crate::codec::{CodecError, MessageCodec};
use crate::messages::Message;
use actix::fut::ready;
use actix::io::WriteHandler;
use actix::{
    io::FramedWrite, Actor, ActorFutureExt, Addr, Context, ContextFutureSpawner, WrapFuture,
};
use actix::{AsyncContext, StreamHandler, WeakAddr};
use log::{debug, error};
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
    tcp_stream_write: FramedWrite<Message, WriteHalf<TcpStream>, MessageCodec>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session").field("id", &self.id).finish()
    }
}

impl Session {
    pub fn new(
        lobby_manager: WeakAddr<LobbyManager>,
        tcp_stream_write: FramedWrite<Message, WriteHalf<TcpStream>, MessageCodec>,
    ) -> Session {
        let s = Session {
            id: get_id(),
            lobby: None,
            lobby_manager,
            tcp_stream_write,
        };
        debug!("Created Session {}", s.id);
        s
    }
}

impl WriteHandler<CodecError> for Session {}

impl Actor for Session {
    type Context = Context<Self>;
}

impl StreamHandler<Result<Message, CodecError>> for Session {
    fn handle(&mut self, item: Result<Message, CodecError>, ctx: &mut Self::Context) {
        debug!("Session {} received {:?}", self.id, item);
        match item {
            Ok(Message::CreateLobby { name }) => {
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
            Ok(Message::ListLobbiesRequest) => {
                let lm_addr = self.lobby_manager.upgrade();
                match lm_addr {
                    Some(a) => {
                        a.send(LobbyManagerMessage::ListLobbies)
                            .into_actor(self)
                            .then(|res, act, ctx| {
                                match res {
                                    Ok(LobbyManagerResponse(Ok(
                                        LobbyManagerResponseSuccess::LobbiesList { lobbies },
                                    ))) => act
                                        .tcp_stream_write
                                        .write(Message::ListLobbiesResponse { lobbies }),
                                    _ => {
                                        error!("FIXME: Give a better error here.");
                                    }
                                }
                                ready(())
                            })
                            .wait(ctx);
                    }
                    None => {
                        error!("Unable to communicate with lobby manager from session");
                    }
                }
            }
            _ => {}
        }
    }
}
