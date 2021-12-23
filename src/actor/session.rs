use crate::actor::lobby::{LobbyMessage, LobbyResponse};
use crate::actor::lobby_manager::LobbyManagerError;
use crate::codec::{CodecError, MessageCodec};
use crate::messages::NetworkMessage;
use actix::fut::ready;
use actix::io::WriteHandler;
use actix::{
    io::FramedWrite, Actor, ActorFutureExt, Context, ContextFutureSpawner, Message, WrapFuture,
};
use actix::{AsyncContext, Handler, Recipient, StreamHandler};
use log::{debug, error};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::WriteHalf;
use tokio::net::TcpStream;

fn get_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

use super::lobby_manager::{LobbyManagerMessage, LobbyManagerResponse};

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub enum SessionMessage {}

pub struct Session {
    id: usize,
    username: Option<String>,
    lobby: Option<Recipient<LobbyMessage>>,
    lobby_manager: Recipient<LobbyManagerMessage>,
    tcp_stream_write: FramedWrite<NetworkMessage, WriteHalf<TcpStream>, MessageCodec>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session").field("id", &self.id).finish()
    }
}

impl Session {
    pub fn new(
        lobby_manager: Recipient<LobbyManagerMessage>,
        tcp_stream_write: FramedWrite<NetworkMessage, WriteHalf<TcpStream>, MessageCodec>,
    ) -> Session {
        let s = Session {
            id: get_id(),
            username: None,
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

impl Handler<SessionMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: SessionMessage, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

impl StreamHandler<Result<NetworkMessage, CodecError>> for Session {
    fn handle(&mut self, item: Result<NetworkMessage, CodecError>, ctx: &mut Self::Context) {
        debug!("Session {} received {:?}", self.id, item);
        match item {
            Ok(NetworkMessage::SetUsername { name }) => {
                self.username = Some(name);
            }
            Ok(NetworkMessage::CreateLobby { name }) => {
                if self.username.is_none() {
                    error!("Cannot create lobby without username");
                    return;
                }

                self.lobby_manager
                    .send(LobbyManagerMessage::CreateLobby {
                        name,
                        creating_session: ctx.address().recipient(),
                    })
                    .into_actor(self)
                    .then(|res, act, _ctx| {
                        match res {
                            Ok(Ok(LobbyManagerResponse::CreatedLobby { lobby })) => {
                                act.lobby = Some(lobby);
                            }
                            _ => error!("Lobby manager returned invalid response for CreateLobby"),
                        }
                        ready(())
                    })
                    .wait(ctx);
            }
            Ok(NetworkMessage::ListLobbiesRequest) => {
                self.lobby_manager
                    .send(LobbyManagerMessage::ListLobbies)
                    .into_actor(self)
                    .then(|res, act, _ctx| {
                        match res {
                            Ok(Ok(LobbyManagerResponse::LobbiesList { lobbies })) => act
                                .tcp_stream_write
                                .write(NetworkMessage::ListLobbiesResponse { lobbies }),
                            _ => {
                                error!("FIXME: Give a better error here. Bad lobby list response.");
                            }
                        }
                        ready(())
                    })
                    .wait(ctx);
            }
            Ok(NetworkMessage::JoinLobby { name }) => {
                if self.username.is_none() {
                    error!("Cannot join lobby without username");
                    return;
                }
                if self.lobby.is_some() {
                    self.tcp_stream_write.write(NetworkMessage::ServerError(
                        "Cannot join lobby when already in lobby.".into(),
                    ));
                    return;
                }
                self.lobby_manager
                    .send(LobbyManagerMessage::ToLobby {
                        name,
                        message: LobbyMessage::JoinLobby(ctx.address().recipient()),
                    })
                    .into_actor(self)
                    .then(|res, act, _ctx| {
                        match res {
                            Ok(Ok(LobbyManagerResponse::LobbyResponse {
                                response: Ok(LobbyResponse::Ok),
                                lobby,
                                name,
                            })) => {
                                act.lobby = Some(lobby);
                                debug!("Session {} joined lobby {}", act.id, name);
                            }
                            Ok(Err(LobbyManagerError::LobbyDoesNotExist)) => {
                                act.tcp_stream_write.write(NetworkMessage::ServerError(
                                    "Unable to join lobby".into(),
                                ));
                                debug!("Session {} unable to join non-existant lobby", act.id);
                            }
                            _ => {
                                error!("FIXME");
                            }
                        }
                        ready(())
                    })
                    .wait(ctx);
            }
            _ => {}
        }
    }
}
