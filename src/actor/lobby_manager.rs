use std::collections::HashMap;

use super::{lobby::Lobby, session::Session};
use actix::{Actor, Addr, Context, Handler, Message, MessageResponse, WeakAddr};
use log::{debug, info};

#[derive(Message, Debug)]
#[rtype(result = "LobbyManagerResponse")]
pub enum LobbyManagerMessage {
    CreateLobby {
        name: String,
        creating_session: Addr<Session>,
    },
    ListLobbies,
}

#[derive(MessageResponse)]
pub struct LobbyManagerResponse(pub Result<LobbyManagerResponseSuccess, LobbyManagerResponseError>);

pub enum LobbyManagerResponseSuccess {
    CreatedLobby { addr: Addr<Lobby> },
    LobbiesList { lobbies: Vec<String> },
}

pub enum LobbyManagerResponseError {
    LobbyNameTaken,
}

#[derive(Default, Debug)]
pub struct LobbyManager {
    lobbies: HashMap<String, WeakAddr<Lobby>>,
}

impl Actor for LobbyManager {
    type Context = Context<Self>;
}

impl Handler<LobbyManagerMessage> for LobbyManager {
    type Result = LobbyManagerResponse;
    fn handle(&mut self, msg: LobbyManagerMessage, _ctx: &mut Self::Context) -> Self::Result {
        debug!("LobbyManager receieved {:?}", msg);
        match msg {
            LobbyManagerMessage::CreateLobby {
                name,
                creating_session,
            } => {
                if self.lobbies.contains_key(&name) {
                    LobbyManagerResponse(Err(LobbyManagerResponseError::LobbyNameTaken))
                } else {
                    let lobby = Lobby::new(creating_session);
                    let addr = Lobby::start(lobby);
                    let n = name.clone();
                    self.lobbies.insert(name, addr.downgrade());
                    info!("Created lobby: {}", n);
                    LobbyManagerResponse(Ok(LobbyManagerResponseSuccess::CreatedLobby { addr }))
                }
            }
            LobbyManagerMessage::ListLobbies => {
                let lobby_names: Vec<String> = self.lobbies.keys().cloned().collect();
                LobbyManagerResponse(Ok(LobbyManagerResponseSuccess::LobbiesList {
                    lobbies: lobby_names,
                }))
            }
        }
    }
}
