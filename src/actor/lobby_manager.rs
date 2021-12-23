use std::collections::HashMap;

use super::{
    lobby::{Lobby, LobbyError, LobbyMessage, LobbyResponse},
    session::SessionMessage,
};
use actix::{Actor, Context, Handler, Message, Recipient, ResponseFuture};
use log::{debug, info};
use thiserror::Error;

#[derive(Message, Debug)]
#[rtype(result = "Result<LobbyManagerResponse, LobbyManagerError>")]
pub enum LobbyManagerMessage {
    CreateLobby {
        name: String,
        creating_session: Recipient<SessionMessage>,
    },
    ListLobbies,
    ToLobby {
        name: String,
        message: LobbyMessage,
    },
}

pub enum LobbyManagerResponse {
    CreatedLobby {
        lobby: Recipient<LobbyMessage>,
    },
    LobbiesList {
        lobbies: Vec<String>,
    },
    LobbyResponse {
        response: Result<LobbyResponse, LobbyError>,
        lobby: Recipient<LobbyMessage>,
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
    lobbies: HashMap<String, Recipient<LobbyMessage>>,
}

impl LobbyManager {
    fn get_lobby(&self, name: &String) -> Result<Recipient<LobbyMessage>, LobbyManagerError> {
        self.lobbies
            .get(name)
            .map(|r| r.clone())
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
                    let recipient = addr.recipient();
                    let n = name.clone();
                    let r = recipient.clone();
                    self.lobbies.insert(name, recipient);
                    info!("Created lobby: {}", n);
                    Box::pin(async { Ok(LobbyManagerResponse::CreatedLobby { lobby: r }) })
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
