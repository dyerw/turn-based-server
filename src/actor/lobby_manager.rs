use std::collections::HashMap;

use super::{
    lobby::{Lobby, LobbyError, LobbyMessage, LobbyResponse},
    session::Session,
};
use actix::{Actor, Addr, Context, Handler, Message, ResponseFuture, WeakAddr, WrapFuture};
use log::{debug, info};
use thiserror::Error;

#[derive(Message, Debug)]
#[rtype(result = "Result<LobbyManagerResponse, LobbyManagerError>")]
pub enum LobbyManagerMessage {
    CreateLobby {
        name: String,
        creating_session: Addr<Session>,
    },
    ListLobbies,
    ToLobby {
        name: String,
        message: LobbyMessage,
    },
}

pub enum LobbyManagerResponse {
    CreatedLobby {
        addr: Addr<Lobby>,
    },
    LobbiesList {
        lobbies: Vec<String>,
    },
    LobbyResponse {
        response: Result<LobbyResponse, LobbyError>,
        lobby: Addr<Lobby>,
        name: String,
    },
}

#[derive(Debug, Error)]
pub enum LobbyManagerError {
    #[error("Lobby name taken")]
    LobbyNameTaken,
    #[error("Lobby does not exist")]
    LobbyDoesNotExist,
    #[error("Error sending message to lobby")]
    MailboxError(#[from] actix::MailboxError),
}

#[derive(Default, Debug)]
pub struct LobbyManager {
    lobbies: HashMap<String, WeakAddr<Lobby>>,
}

impl LobbyManager {
    fn get_lobby(&self, name: &String) -> Result<Addr<Lobby>, LobbyManagerError> {
        self.lobbies
            .get(name)
            .and_then(|o| o.upgrade())
            .ok_or(LobbyManagerError::LobbyDoesNotExist)
    }
}

impl Actor for LobbyManager {
    type Context = Context<Self>;
}

impl Handler<LobbyManagerMessage> for LobbyManager {
    type Result = ResponseFuture<Result<LobbyManagerResponse, LobbyManagerError>>;
    fn handle(&mut self, msg: LobbyManagerMessage, _ctx: &mut Self::Context) -> Self::Result {
        debug!("LobbyManager receieved {:?}", msg);
        match msg {
            LobbyManagerMessage::CreateLobby {
                name,
                creating_session,
            } => {
                if self.lobbies.contains_key(&name) {
                    Box::pin(async { Err(LobbyManagerError::LobbyNameTaken) })
                } else {
                    let lobby = Lobby::new(creating_session);
                    let addr = Lobby::start(lobby);
                    let n = name.clone();
                    self.lobbies.insert(name, addr.downgrade());
                    info!("Created lobby: {}", n);
                    Box::pin(async { Ok(LobbyManagerResponse::CreatedLobby { addr }) })
                }
            }
            LobbyManagerMessage::ListLobbies => {
                let lobby_names: Vec<String> = self.lobbies.keys().cloned().collect();
                Box::pin(async {
                    Ok(LobbyManagerResponse::LobbiesList {
                        lobbies: lobby_names,
                    })
                })
            }
            LobbyManagerMessage::ToLobby { name, message } => {
                let lobby_result = self.get_lobby(&name);

                Box::pin(async {
                    let lobby = lobby_result?;
                    let response = lobby.send(message).await?;
                    Ok(LobbyManagerResponse::LobbyResponse {
                        response,
                        lobby,
                        name,
                    })
                })
            }
        }
    }
}
